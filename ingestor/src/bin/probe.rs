use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Days, NaiveDate, Utc};
use clap::Parser;
use ingestor::fetch::{distinct, fetch_snapshot, query_chunked};
use ingestor::leaguepedia::{Client, Query, Row, in_list};
use serde_json::json;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    about = "Dump recent Leaguepedia roster changes for LCK, LPL, LEC and LCS to JSON fixtures"
)]
struct Cli {
    /// First roster change date to fetch (YYYY-MM-DD). Defaults to 31 days ago.
    #[arg(long)]
    since: Option<NaiveDate>,

    /// Output directory for the fixtures.
    #[arg(long, default_value = "tests/fixtures/probe")]
    out: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let today = Utc::now().date_naive();
    let since = cli.since.unwrap_or(today - Days::new(31));
    std::fs::create_dir_all(&cli.out)?;
    let mut client = Client::new()?;

    let (snapshot, _) = fetch_snapshot(&mut client, since).await?;
    let tables = [
        ("tournaments", &snapshot.tournaments),
        ("tournament_rosters", &snapshot.tournament_rosters),
        ("team_redirects", &snapshot.team_redirects),
        ("teams", &snapshot.teams),
        ("roster_changes", &snapshot.roster_changes),
        ("player_redirects", &snapshot.player_redirects),
        ("players", &snapshot.players),
    ];
    for (name, rows) in tables {
        write(&cli.out, name, rows)?;
    }

    let tenures = query_chunked(
        &mut client,
        &distinct(&snapshot.roster_changes, "RosterChangeId"),
        |chunk| {
            Ok(Query {
                tables: "Tenures".into(),
                fields: "Player,Team,DateJoin,DateLeave,Duration,ContractEnd,RosterChangeIdJoin,\
                         RosterChangeIdLeave,ResidencyLeave,NameLeave,NextTeam,NextIsRetired,\
                         IsCurrent"
                    .into(),
                where_clause: format!(
                    "{} OR {}",
                    in_list("RosterChangeIdJoin", chunk)?,
                    in_list("RosterChangeIdLeave", chunk)?
                ),
                order_by: "Player,DateJoin".into(),
            })
        },
    )
    .await?;
    write(&cli.out, "tenures", &tenures)?;

    let mut counts: serde_json::Map<String, serde_json::Value> = tables
        .iter()
        .map(|(name, rows)| ((*name).to_owned(), rows.len().into()))
        .collect();
    counts.insert("tenures".into(), tenures.len().into());
    let meta = json!({
        "source": "https://lol.fandom.com (Leaguepedia, CC BY-SA 3.0)",
        "fetched_on": today.to_string(),
        "since": since.to_string(),
        "counts": counts,
    });
    std::fs::write(
        cli.out.join("meta.json"),
        serde_json::to_string_pretty(&meta)? + "\n",
    )?;
    tracing::info!(counts = %meta["counts"], out = %cli.out.display(), "probe finished");
    Ok(())
}

fn write(dir: &Path, name: &str, rows: &[Row]) -> Result<()> {
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(rows)? + "\n")
        .with_context(|| format!("writing {}", path.display()))?;
    tracing::info!(file = %path.display(), rows = rows.len(), "wrote fixture");
    Ok(())
}
