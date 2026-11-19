use std::path::Path;

use ingestor::leaguepedia::Row;
use ingestor::normalize::Snapshot;

#[allow(dead_code)]
pub fn load_probe(name: &str) -> Vec<Row> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/probe")
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

pub fn probe_snapshot() -> Snapshot {
    let load = load_probe;
    Snapshot {
        tournaments: load("tournaments"),
        tournament_rosters: load("tournament_rosters"),
        team_redirects: load("team_redirects"),
        teams: load("teams"),
        roster_changes: load("roster_changes"),
        player_redirects: load("player_redirects"),
        players: load("players"),
        player_images: load("player_images"),
    }
}
