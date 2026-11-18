mod common;

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use ingestor::leaguepedia::Row;
use ingestor::normalize::{Normalized, normalize};
use ingestor::pairing::{Kind, PAIR_WINDOW_DAYS, Transfer, pair};
use serde_json::Value;

fn text<'a>(row: &'a Row, key: &str) -> &'a str {
    row.get(key).and_then(Value::as_str).unwrap_or("")
}

fn probe() -> (Normalized, Vec<Transfer>) {
    let data = normalize(&common::probe_snapshot()).unwrap();
    let renames = data
        .teams
        .iter()
        .filter_map(|t| Some((t.name.clone(), t.renamed_to.clone()?)))
        .collect();
    let transfers = pair(&data.raw_events, &renames);
    (data, transfers)
}

fn canonical<'a>(aliases: &HashMap<&str, &'a str>, name: &'a str) -> &'a str {
    aliases.get(name).copied().unwrap_or(name)
}

fn date(row: &Row, key: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text(row, key), "%Y-%m-%d").ok()
}

/// `Tenures.NextTeam` is Leaguepedia's own answer to "where did the player go after this leave",
/// maintained independently of `RosterChanges`, so it checks every leave we pair. A next team
/// joined more than the pairing window later is a release followed by a signing, and one whose
/// join is missing from `RosterChanges` cannot be paired at all, so neither is counted.
#[test]
fn leaves_agree_with_leaguepedia_tenures() {
    let (data, transfers) = probe();
    let redirects = common::load_probe("team_redirects");
    let team_aliases: HashMap<&str, &str> = redirects
        .iter()
        .map(|row| (text(row, "AllName"), text(row, "Page")))
        .collect();
    let tenures = common::load_probe("tenures");
    let events: HashSet<&str> = data.raw_events.iter().map(|e| e.id.as_str()).collect();
    let mut next_team: HashMap<&str, Option<&str>> = HashMap::new();
    for row in &tenures {
        let leave = text(row, "RosterChangeIdLeave");
        let next = text(row, "NextTeam");
        if leave.is_empty() {
            continue;
        }
        if next.is_empty() {
            next_team.insert(leave, None);
            continue;
        }
        let left = date(row, "DateLeave");
        let joined = tenures
            .iter()
            .filter(|t| {
                text(t, "Player") == text(row, "Player")
                    && canonical(&team_aliases, text(t, "Team")) == canonical(&team_aliases, next)
            })
            .filter(|t| date(t, "DateJoin") >= left)
            .min_by_key(|t| date(t, "DateJoin"));
        let Some(joined) = joined else {
            continue;
        };
        if !events.contains(text(joined, "RosterChangeIdJoin")) {
            continue;
        }
        let in_window = match (left, date(joined, "DateJoin")) {
            (Some(left), Some(join)) => (join - left).num_days() <= PAIR_WINDOW_DAYS,
            _ => false,
        };
        next_team.insert(leave, in_window.then(|| canonical(&team_aliases, next)));
    }

    let mut checked = 0;
    let mut wrong = Vec::new();
    for transfer in &transfers {
        let Some(leave) = transfer.leave_event.as_deref() else {
            continue;
        };
        let Some(expected) = next_team.get(leave) else {
            continue;
        };
        if matches!(transfer.kind, Kind::Bench | Kind::Sub) {
            continue;
        }
        checked += 1;
        if transfer.to_team.as_deref() != *expected {
            wrong.push(format!(
                "{leave}: paired {:?}, tenures say {expected:?}",
                transfer.to_team
            ));
        }
    }
    let correct = checked - wrong.len();
    assert!(checked >= 50, "only {checked} leaves have a tenure");
    assert!(
        correct * 100 >= checked * 95,
        "{correct}/{checked} correct:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn every_live_event_is_used_at_most_once() {
    let (data, transfers) = probe();
    let mut seen = HashMap::new();
    for transfer in &transfers {
        for id in [&transfer.leave_event, &transfer.join_event]
            .into_iter()
            .flatten()
        {
            assert!(seen.insert(id.clone(), ()).is_none(), "{id} used twice");
        }
    }
    let ids: Vec<&str> = data.raw_events.iter().map(|e| e.id.as_str()).collect();
    assert!(seen.keys().all(|id| ids.contains(&id.as_str())));
}
