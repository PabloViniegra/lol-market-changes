mod common;

use std::collections::BTreeSet;

use chrono::NaiveDate;
use ingestor::leaguepedia::Row;
use ingestor::model::Direction;
use ingestor::normalize::{Normalized, Skipped, Snapshot, normalize};
use serde_json::{Value, json};

fn probe() -> Normalized {
    normalize(&common::probe_snapshot()).unwrap()
}

#[test]
fn every_event_references_a_known_player_and_team() {
    let data = probe();
    let players: BTreeSet<&str> = data.players.iter().map(|p| p.name.as_str()).collect();
    let teams: BTreeSet<&str> = data.teams.iter().map(|t| t.name.as_str()).collect();
    for event in &data.raw_events {
        assert!(players.contains(event.player.as_str()), "{}", event.id);
        assert!(teams.contains(event.team.as_str()), "{}", event.id);
    }
    for season in &data.team_seasons {
        assert!(
            teams.contains(season.team.as_str()),
            "{}",
            season.tournament
        );
    }
}

#[test]
fn event_ids_are_unique() {
    let data = probe();
    let ids: BTreeSet<&str> = data.raw_events.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids.len(), data.raw_events.len());
}

#[test]
fn staff_and_game_changers_rows_are_skipped() {
    let data = probe();
    assert!(data.skipped.staff > 0);
    assert!(data.raw_events.iter().all(|e| e.role.is_some()));
}

#[test]
fn renamed_teams_resolve_to_their_current_page() {
    let data = probe();
    let teams: BTreeSet<&str> = data.teams.iter().map(|t| t.name.as_str()).collect();
    for (old, current) in [("DRX", "Kiwoom DRX"), ("BRION", "HANJIN BRION")] {
        assert!(!teams.contains(old), "{old} should resolve to {current}");
        assert!(teams.contains(current), "{current}");
    }
}

#[test]
fn lowercase_player_names_resolve_to_the_page_title() {
    let data = probe();
    assert!(data.players.iter().all(|p| p.name != "kyeahoo"));
    assert!(data.players.iter().any(|p| p.name == "Kyeahoo"));
}

#[test]
fn keeps_a_known_signing() {
    let data = probe();
    let event = data
        .raw_events
        .iter()
        .find(|e| e.player == "Ucal" && e.team == "DN SOOPers" && e.direction == Direction::Join)
        .expect("Ucal joining DN SOOPers");
    assert_eq!(event.date, NaiveDate::from_ymd_opt(2026, 11, 18).unwrap());
    assert_eq!(event.role.as_deref(), Some("Mid"));
    assert!(!event.is_sub && !event.is_loan && !event.is_retired);
    let team = data.teams.iter().find(|t| t.name == "DN SOOPers").unwrap();
    assert_eq!(team.league, "LCK");
}

fn row(value: Value) -> Row {
    value.as_object().unwrap().clone()
}

fn change(id: &str, roles: &str, preload: &str, tags: &str, is_gcd: &str) -> Row {
    row(json!({
        "RosterChangeId": id, "Date": "2026-11-18", "Player": "Faker", "Team": "T1",
        "Direction": "Join", "RolesIngame": roles, "Preload": preload, "Tags": tags,
        "IsGCD": is_gcd,
    }))
}

#[test]
fn skips_game_changers_team_renames_and_staff() {
    let snapshot = Snapshot {
        tournaments: vec![row(json!({
            "OverviewPage": "LCK/2026 Season", "League": "LoL Champions Korea",
            "DateStart": "2026-01-14", "Date": "",
        }))],
        tournament_rosters: vec![row(
            json!({"OverviewPage": "LCK/2026 Season", "Team": "T1"}),
        )],
        roster_changes: vec![
            change("player", "Mid", "join", "", "0"),
            change("gcd", "Mid", "join", "", "1"),
            change("tagged_rename", "Mid", "join", "TeamRename", "0"),
            change("preload_rename", "Mid", "team_rename_join", "", "0"),
            change("staff", "", "join", "", "0"),
        ],
        ..Default::default()
    };
    let data = normalize(&snapshot).unwrap();
    let ids: Vec<&str> = data.raw_events.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, ["player"]);
    assert_eq!(
        data.skipped,
        Skipped {
            staff: 1,
            game_changers: 1,
            team_renames: 2,
            out_of_scope: 0,
        }
    );
}

#[test]
fn output_is_deterministic() {
    let (a, b) = (probe(), probe());
    assert_eq!(a.raw_events, b.raw_events);
    assert_eq!(a.players, b.players);
    assert_eq!(a.teams, b.teams);
    assert_eq!(a.team_seasons, b.team_seasons);
}
