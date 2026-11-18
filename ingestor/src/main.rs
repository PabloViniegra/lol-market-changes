use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(version, about = "Leaguepedia transfer market ingestor")]
struct Cli {
    /// Backfill instead of an incremental fetch.
    #[arg(long, conflicts_with = "since")]
    full: bool,

    /// Fetch roster changes on or after this date (YYYY-MM-DD).
    #[arg(long)]
    since: Option<NaiveDate>,

    /// Fetch and process without writing to the database.
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    tracing::info!(full = cli.full, since = ?cli.since, dry_run = cli.dry_run, "ingestor started");
    tracing::warn!("ingestion pipeline not implemented yet");
    Ok(())
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
}
