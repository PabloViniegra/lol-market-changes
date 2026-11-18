use std::path::Path;

use ingestor::leaguepedia::Row;
use ingestor::normalize::Snapshot;

pub fn probe_snapshot() -> Snapshot {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/probe");
    let load = |name: &str| -> Vec<Row> {
        let path = dir.join(format!("{name}.json"));
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        serde_json::from_str(&text).unwrap()
    };
    Snapshot {
        tournaments: load("tournaments"),
        tournament_rosters: load("tournament_rosters"),
        team_redirects: load("team_redirects"),
        teams: load("teams"),
        roster_changes: load("roster_changes"),
        player_redirects: load("player_redirects"),
        players: load("players"),
    }
}
