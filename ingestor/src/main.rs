use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::{Datelike, Days, NaiveDate, Utc};
use clap::Parser;
use ingestor::db::{self, Window};
use ingestor::fetch::{Scope, ScopeRows, fetch_scope, fetch_snapshot};
use ingestor::leaguepedia::Client;
use ingestor::normalize::{Normalized, normalize};
use ingestor::rumors::{self, Rumor};
use serde_json::{Value, json};
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

// Re-fetch this many days before the last sync to pick up retroactive wiki edits.
const OVERLAP_DAYS: u64 = 14;

#[derive(Debug, Parser)]
#[command(version, about = "Leaguepedia transfer market ingestor")]
struct Cli {
    /// Backfill the last two seasons instead of an incremental fetch.
    #[arg(long, conflicts_with = "since")]
    full: bool,

    /// Fetch roster changes on or after this date (YYYY-MM-DD).
    #[arg(long)]
    since: Option<NaiveDate>,

    /// Fetch and process without connecting to the database.
    #[arg(long)]
    dry_run: bool,

    /// Allow soft-deleting most of the re-fetched window, after checking the wiki removed the rows.
    #[arg(long)]
    allow_mass_delete: bool,

    /// Hand-curated rumors file.
    #[arg(long, default_value = "../data/rumors.yaml")]
    rumors: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let today = Utc::now().date_naive();

    if cli.dry_run {
        let rumors = load_rumors(&cli.rumors)?;
        let since = cli
            .since
            .or(cli.full.then(|| backfill_start(today)))
            .unwrap_or(today - Days::new(OVERLAP_DAYS));
        let mut client = Client::new()?;
        let scope_rows = fetch_scope(&mut client, since).await?;
        let (data, _) = fetch_and_normalize(&mut client, since, scope_rows).await?;
        tracing::info!(stats = %normalized_stats(&data), rumors = rumors.len(), "dry run finished");
        return Ok(());
    }

    let url = std::env::var("INGESTOR_DATABASE_URL").context("INGESTOR_DATABASE_URL is not set")?;
    let pool = db::connect(&url).await?;
    let result = run(&pool, &cli, today).await;
    if let Err(error) = &result {
        tracing::error!(error = format!("{error:#}"), "ingestion failed");
    }
    let (status, stats) = match &result {
        Ok(stats) => ("ok", stats.clone()),
        Err(error) => ("error", json!({ "error": format!("{error:#}") })),
    };
    let synced = result.is_ok().then_some(today);
    db::record_run(&pool, synced, status, &stats).await?;
    result.map(|_| ())
}

async fn run(pool: &PgPool, cli: &Cli, today: NaiveDate) -> Result<Value> {
    let rumors = load_rumors(&cli.rumors)?;
    let since = match (cli.since, cli.full) {
        (Some(since), _) => since,
        (None, true) => backfill_start(today),
        (None, false) => db::last_synced_date(pool)
            .await?
            .map(|date| date - Days::new(OVERLAP_DAYS))
            .unwrap_or_else(|| backfill_start(today)),
    };
    let started = Instant::now();
    let mut client = Client::new()?;
    let cached = match cli.full {
        true => None,
        false => db::load_scope_rows(pool, since).await?,
    };
    let scope_cached = cached.is_some();
    let scope_rows = match cached {
        Some(rows) => rows,
        None => {
            let rows = fetch_scope(&mut client, since).await?;
            db::save_scope_rows(pool, since, &rows).await?;
            rows
        }
    };
    let (data, scope) = fetch_and_normalize(&mut client, since, scope_rows).await?;
    let window = Window {
        since,
        players: &scope.players,
        teams: &scope.teams,
        allow_mass_delete: cli.allow_mass_delete,
    };
    let written = db::write(pool, &data, &window).await?;
    let rumors_changed = db::sync_rumors(pool, &rumors, today).await?;
    let stats = json!({
        "since": since.to_string(),
        "duration_secs": started.elapsed().as_secs(),
        "scope_cached": scope_cached,
        "fetched": normalized_stats(&data),
        "changed": {
            "teams": written.teams,
            "team_seasons": written.team_seasons,
            "players": written.players,
            "raw_events": written.raw_events,
            "deleted_events": written.deleted_events,
            "transfers": written.transfers,
            "rumors": rumors_changed,
        },
    });
    tracing::info!(%stats, "ingestion finished");
    Ok(stats)
}

fn load_rumors(path: &Path) -> Result<Vec<Rumor>> {
    let yaml =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    rumors::parse(&yaml, rumors::latest_report_date(Utc::now()))
}

async fn fetch_and_normalize(
    client: &mut Client,
    since: NaiveDate,
    scope_rows: ScopeRows,
) -> Result<(Normalized, Scope)> {
    tracing::info!(%since, "fetching Leaguepedia");
    let (snapshot, scope) = fetch_snapshot(client, since, scope_rows).await?;
    Ok((normalize(&snapshot)?, scope))
}

fn normalized_stats(data: &Normalized) -> Value {
    json!({
        "teams": data.teams.len(),
        "team_logos": data.teams.iter().filter(|team| team.logo_url.is_some()).count(),
        "team_seasons": data.team_seasons.len(),
        "players": data.players.len(),
        "raw_events": data.raw_events.len(),
        "skipped": {
            "staff": data.skipped.staff,
            "game_changers": data.skipped.game_changers,
            "team_renames": data.skipped.team_renames,
            "out_of_scope": data.skipped.out_of_scope,
        },
    })
}

/// Start of the off-season before the previous season, e.g. 2024-11-01 when run in 2026.
fn backfill_start(today: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(today.year() - 2, 11, 1).expect("November 1st is a valid date")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn full_and_since_are_mutually_exclusive() {
        assert!(Cli::try_parse_from(["ingestor", "--full", "--since", "2026-01-01"]).is_err());
    }

    #[test]
    fn backfill_covers_two_seasons_and_their_off_seasons() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 18).unwrap();
        assert_eq!(
            backfill_start(today),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap()
        );
    }
}
