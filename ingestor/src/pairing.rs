use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;

use crate::model::Direction;
use crate::normalize::RawEvent;

pub const PAIR_WINDOW_DAYS: i64 = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Transfer,
    FreeAgentSigning,
    Release,
    Loan,
    Promotion,
    Bench,
    Sub,
    Retirement,
}

impl Kind {
    pub const ALL: [Kind; 8] = [
        Self::Transfer,
        Self::FreeAgentSigning,
        Self::Release,
        Self::Loan,
        Self::Promotion,
        Self::Bench,
        Self::Sub,
        Self::Retirement,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transfer => "transfer",
            Self::FreeAgentSigning => "free_agent_signing",
            Self::Release => "release",
            Self::Loan => "loan",
            Self::Promotion => "promotion",
            Self::Bench => "bench",
            Self::Sub => "sub",
            Self::Retirement => "retirement",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub player: String,
    pub from_team: Option<String>,
    pub to_team: Option<String>,
    pub role: Option<String>,
    pub kind: Kind,
    pub date: NaiveDate,
    pub leave_event: Option<String>,
    pub join_event: Option<String>,
}

/// Rebuilds the moves of every player in `events` (§6.3). `renames` maps a team page to the
/// page it was renamed to (`Teams.RenamedTo`), so both pages count as one team.
pub fn pair(events: &[RawEvent], renames: &HashMap<String, String>) -> Vec<Transfer> {
    let mut by_player: BTreeMap<&str, Vec<&RawEvent>> = BTreeMap::new();
    for event in events {
        by_player.entry(&event.player).or_default().push(event);
    }
    let lineage = |team: &str| lineage(renames, team);
    let mut transfers: Vec<Transfer> = by_player
        .into_values()
        .flat_map(|mut events| {
            events.sort_by(|a, b| order_key(a).cmp(&order_key(b)));
            pair_player(&steps(&events, &lineage), &lineage)
        })
        .collect();
    transfers.sort_by(|a, b| {
        (a.date, &a.player, &a.leave_event, &a.join_event).cmp(&(
            b.date,
            &b.player,
            &b.leave_event,
            &b.join_event,
        ))
    });
    transfers
}

enum Step<'a> {
    Leave(&'a RawEvent),
    Join(&'a RawEvent),
    /// Same-day leave and join on one team: a contract, role, bench or loan change, not a move.
    Stay(&'a RawEvent, &'a RawEvent),
}

fn order_key(event: &RawEvent) -> impl Ord + '_ {
    (
        event.date,
        event.direction == Direction::Join,
        event.news_id.as_deref(),
        event.line_in_news,
        event.id.as_str(),
    )
}

fn steps<'a>(events: &[&'a RawEvent], lineage: &impl Fn(&str) -> String) -> Vec<Step<'a>> {
    let mut used = vec![false; events.len()];
    let mut steps = Vec::new();
    for (i, event) in events.iter().enumerate() {
        if used[i] {
            continue;
        }
        used[i] = true;
        if event.direction == Direction::Join {
            steps.push(Step::Join(event));
            continue;
        }
        let team = lineage(&event.team);
        let partner = (i + 1..events.len()).find(|&k| {
            !used[k]
                && events[k].date == event.date
                && events[k].direction == Direction::Join
                && lineage(&events[k].team) == team
        });
        match partner {
            Some(k) => {
                used[k] = true;
                steps.push(Step::Stay(event, events[k]));
            }
            None => steps.push(Step::Leave(event)),
        }
    }
    steps
}

fn pair_player(steps: &[Step], lineage: &impl Fn(&str) -> String) -> Vec<Transfer> {
    let mut transfers = Vec::new();
    // A player can sit on two rosters at once (league team plus academy or college team), so
    // several leaves may wait for the same join.
    let mut pending: Vec<&RawEvent> = Vec::new();
    let mut loan_origin: Option<&RawEvent> = None;
    // Last direction per team, so a confirmation of a join or leave already seen is skipped.
    let mut last: HashMap<String, Direction> = HashMap::new();
    let mut repeats = |event: &RawEvent| {
        last.insert(lineage(&event.team), event.direction) == Some(event.direction)
    };
    let release = |transfers: &mut Vec<Transfer>, leaves: &mut Vec<&RawEvent>| {
        transfers.extend(
            leaves
                .drain(..)
                .map(|leave| transfer(Kind::Release, Some(leave), None)),
        );
    };
    for step in steps {
        match *step {
            Step::Stay(leave, join) => {
                if leave.is_sub != join.is_sub {
                    let kind = if join.is_sub { Kind::Bench } else { Kind::Sub };
                    transfers.push(transfer(kind, Some(leave), Some(join)));
                } else if leave.is_loan || join.is_loan {
                    let borrower = pending
                        .iter()
                        .position(|p| p.is_loan && within_window(p, join));
                    match borrower {
                        Some(i) => {
                            let borrower = pending.remove(i);
                            transfers.push(transfer(Kind::Loan, Some(borrower), Some(join)));
                        }
                        None => loan_origin = Some(leave),
                    }
                }
            }
            Step::Leave(leave) => {
                if is_not_a_move(leave) || repeats(leave) {
                    continue;
                }
                if leave.is_retired {
                    transfers.push(transfer(Kind::Retirement, Some(leave), None));
                } else {
                    pending.push(leave);
                }
            }
            Step::Join(join) => {
                if is_not_a_move(join) || repeats(join) {
                    continue;
                }
                let (mut open, mut expired): (Vec<_>, Vec<_>) = pending
                    .drain(..)
                    .partition(|leave| within_window(leave, join));
                release(&mut transfers, &mut expired);
                let origin = open
                    .iter()
                    .position(|leave| !leave.is_sub)
                    .or((!open.is_empty()).then_some(0))
                    .map(|i| open.remove(i));
                release(&mut transfers, &mut open);
                if let Some(leave) = origin {
                    if lineage(&leave.team) != lineage(&join.team) {
                        transfers.push(transfer(pair_kind(leave, join), Some(leave), Some(join)));
                    }
                    continue;
                }
                let lender = loan_origin.filter(|origin| {
                    join.is_loan
                        && within_window(origin, join)
                        && lineage(&origin.team) != lineage(&join.team)
                });
                match lender {
                    Some(origin) => transfers.push(transfer(Kind::Loan, Some(origin), Some(join))),
                    None => transfers.push(transfer(Kind::FreeAgentSigning, None, Some(join))),
                }
            }
        }
    }
    release(&mut transfers, &mut pending);
    transfers
}

fn is_not_a_move(event: &RawEvent) -> bool {
    matches!(event.preload.as_str(), "extended" | "remain")
        || event.preload.starts_with("role_swap")
        || event.status.as_deref() == Some("temp_sub")
}

fn within_window(earlier: &RawEvent, later: &RawEvent) -> bool {
    (later.date - earlier.date).num_days() <= PAIR_WINDOW_DAYS
}

fn pair_kind(leave: &RawEvent, join: &RawEvent) -> Kind {
    if join.is_loan {
        Kind::Loan
    } else if leave.preload == "to_main" || join.preload == "from_academy" {
        Kind::Promotion
    } else {
        Kind::Transfer
    }
}

fn transfer(kind: Kind, leave: Option<&RawEvent>, join: Option<&RawEvent>) -> Transfer {
    let main = join.or(leave).expect("a transfer has at least one event");
    Transfer {
        player: main.player.clone(),
        from_team: leave.map(|e| e.team.clone()),
        to_team: join.map(|e| e.team.clone()),
        role: join
            .and_then(|e| e.role.clone())
            .or_else(|| leave.and_then(|e| e.role.clone())),
        kind,
        date: main.date,
        leave_event: leave.map(|e| e.id.clone()),
        join_event: join.map(|e| e.id.clone()),
    }
}

pub fn lineage(renames: &HashMap<String, String>, team: &str) -> String {
    let mut current = team;
    let mut seen = HashSet::from([team]);
    while let Some(next) = renames.get(current) {
        if !seen.insert(next) {
            return team.to_owned();
        }
        current = next;
    }
    current.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str, date: &str, team: &str, direction: Direction) -> RawEvent {
        RawEvent {
            id: id.to_owned(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            player: "Faker".to_owned(),
            player_raw: "Faker".to_owned(),
            team: team.to_owned(),
            team_raw: team.to_owned(),
            direction,
            role: Some("Mid".to_owned()),
            preload: match direction {
                Direction::Join => "join",
                Direction::Leave => "leave",
            }
            .to_owned(),
            status: None,
            is_sub: false,
            is_loan: false,
            is_retired: false,
            news_id: None,
            line_in_news: None,
            source_url: None,
        }
    }

    fn leave(id: &str, date: &str, team: &str) -> RawEvent {
        event(id, date, team, Direction::Leave)
    }

    fn join(id: &str, date: &str, team: &str) -> RawEvent {
        event(id, date, team, Direction::Join)
    }

    fn kinds(transfers: &[Transfer]) -> Vec<(Kind, Option<&str>, Option<&str>)> {
        transfers
            .iter()
            .map(|t| (t.kind, t.from_team.as_deref(), t.to_team.as_deref()))
            .collect()
    }

    fn run(events: &[RawEvent]) -> Vec<Transfer> {
        pair(events, &HashMap::new())
    }

    #[test]
    fn pairs_a_leave_with_the_next_join_into_a_transfer() {
        let transfers = run(&[
            leave("l", "2025-11-18", "T1"),
            join("j", "2025-11-20", "Gen.G"),
        ]);
        assert_eq!(
            transfers,
            [Transfer {
                player: "Faker".to_owned(),
                from_team: Some("T1".to_owned()),
                to_team: Some("Gen.G".to_owned()),
                role: Some("Mid".to_owned()),
                kind: Kind::Transfer,
                date: NaiveDate::from_ymd_opt(2025, 11, 20).unwrap(),
                leave_event: Some("l".to_owned()),
                join_event: Some("j".to_owned()),
            }]
        );
    }

    fn with(mut event: RawEvent, edit: impl FnOnce(&mut RawEvent)) -> RawEvent {
        edit(&mut event);
        event
    }

    #[test]
    fn a_join_without_a_pending_leave_is_a_free_agent_signing() {
        let transfers = run(&[join("j", "2025-11-20", "Gen.G")]);
        assert_eq!(
            kinds(&transfers),
            [(Kind::FreeAgentSigning, None, Some("Gen.G"))]
        );
        assert_eq!(transfers[0].leave_event, None);
        assert_eq!(transfers[0].join_event.as_deref(), Some("j"));
    }

    #[test]
    fn an_unmatched_leave_is_a_release_or_a_retirement() {
        let transfers = run(&[leave("l", "2025-11-18", "T1")]);
        assert_eq!(kinds(&transfers), [(Kind::Release, Some("T1"), None)]);

        let retired = with(leave("r", "2025-11-18", "T1"), |e| {
            e.preload = "retirement".to_owned();
            e.is_retired = true;
        });
        assert_eq!(
            kinds(&run(&[retired])),
            [(Kind::Retirement, Some("T1"), None)]
        );
    }

    #[test]
    fn a_retirement_is_never_paired_with_a_later_join() {
        let retired = with(leave("r", "2025-11-18", "T1"), |e| e.is_retired = true);
        let transfers = run(&[retired, join("j", "2025-12-01", "Gen.G")]);
        assert_eq!(
            kinds(&transfers),
            [
                (Kind::Retirement, Some("T1"), None),
                (Kind::FreeAgentSigning, None, Some("Gen.G")),
            ]
        );
    }

    #[test]
    fn a_join_outside_the_window_is_not_paired() {
        let transfers = run(&[
            leave("l", "2025-11-01", "T1"),
            join("j", "2025-12-17", "Gen.G"),
        ]);
        assert_eq!(
            kinds(&transfers),
            [
                (Kind::Release, Some("T1"), None),
                (Kind::FreeAgentSigning, None, Some("Gen.G")),
            ]
        );
        let inside = run(&[
            leave("l", "2025-11-01", "T1"),
            join("j", "2025-12-16", "Gen.G"),
        ]);
        assert_eq!(
            kinds(&inside),
            [(Kind::Transfer, Some("T1"), Some("Gen.G"))]
        );
    }

    #[test]
    fn same_day_events_put_the_leave_first_regardless_of_input_order() {
        let transfers = run(&[
            join("j", "2025-11-18", "Gen.G"),
            leave("l", "2025-11-18", "T1"),
        ]);
        assert_eq!(
            kinds(&transfers),
            [(Kind::Transfer, Some("T1"), Some("Gen.G"))]
        );
    }

    #[test]
    fn a_player_leaving_two_teams_moves_from_the_first_and_is_released_from_the_other() {
        let transfers = run(&[
            leave("l1", "2025-11-18", "DarkZero"),
            leave("l2", "2025-11-19", "Maryville University"),
            join("j", "2025-11-25", "G2 NORD"),
        ]);
        assert_eq!(
            kinds(&transfers),
            [
                (Kind::Release, Some("Maryville University"), None),
                (Kind::Transfer, Some("DarkZero"), Some("G2 NORD")),
            ]
        );
    }

    #[test]
    fn a_starting_spot_is_preferred_over_a_sub_spot_as_the_origin() {
        let transfers = run(&[
            with(leave("l1", "2025-12-03", "Milk Esports"), |e| {
                e.is_sub = true
            }),
            leave("l2", "2025-12-03", "Maryville University"),
            join("j", "2025-12-03", "Disguised"),
        ]);
        assert_eq!(
            kinds(&transfers),
            [
                (Kind::Release, Some("Milk Esports"), None),
                (
                    Kind::Transfer,
                    Some("Maryville University"),
                    Some("Disguised")
                ),
            ]
        );
    }

    #[test]
    fn signing_elsewhere_after_a_loan_ends_is_a_transfer() {
        let transfers = run(&[
            loan(
                leave("bl", "2025-11-17", "BRION"),
                "loan_end",
                Some("on_loan"),
            ),
            join("j", "2025-11-20", "Misa Esports"),
        ]);
        assert_eq!(
            kinds(&transfers),
            [(Kind::Transfer, Some("BRION"), Some("Misa Esports"))]
        );
    }

    #[test]
    fn same_day_leave_and_join_on_one_team_is_not_a_move() {
        let extended = |e: &mut RawEvent| e.preload = "extended".to_owned();
        let transfers = run(&[
            with(leave("l", "2025-11-18", "T1"), extended),
            with(join("j", "2025-11-18", "T1"), extended),
        ]);
        assert_eq!(transfers, []);

        let swap = |e: &mut RawEvent| e.preload = "role_swap".to_owned();
        let transfers = run(&[
            with(leave("l", "2025-11-18", "T1"), swap),
            with(join("j", "2025-11-18", "T1"), |e| {
                swap(e);
                e.role = Some("Top".to_owned());
            }),
        ]);
        assert_eq!(transfers, []);
    }

    #[test]
    fn an_unpaired_extension_is_not_a_move() {
        let transfers = run(&[with(leave("l", "2025-11-18", "T1"), |e| {
            e.preload = "extended".to_owned();
        })]);
        assert_eq!(transfers, []);
    }

    #[test]
    fn temporary_sub_call_ups_are_not_moves() {
        let temp = |mut e: RawEvent, preload: &str| {
            e.preload = preload.to_owned();
            e.status = Some("temp_sub".to_owned());
            e
        };
        let transfers = run(&[
            temp(join("j", "2025-06-01", "T1"), "from_academy_as_temp_sub"),
            temp(leave("l", "2025-06-20", "T1"), "to_academy"),
        ]);
        assert_eq!(transfers, []);
    }

    #[test]
    fn confirmations_of_a_known_move_are_not_new_moves() {
        let transfers = run(&[
            leave("l", "2025-11-18", "T1"),
            join("j", "2025-11-20", "Gen.G"),
            with(join("cj", "2025-11-25", "Gen.G"), |e| {
                e.preload = "confirm_join".to_owned();
            }),
            with(leave("cl", "2025-11-26", "T1"), |e| {
                e.preload = "confirm_leave".to_owned();
            }),
        ]);
        assert_eq!(
            kinds(&transfers),
            [(Kind::Transfer, Some("T1"), Some("Gen.G"))]
        );
    }

    #[test]
    fn rejoining_the_same_team_within_the_window_is_not_a_move() {
        let transfers = run(&[
            leave("l", "2025-11-18", "T1"),
            join("j", "2025-12-01", "T1"),
        ]);
        assert_eq!(transfers, []);
    }

    #[test]
    fn moving_to_or_from_the_bench_on_one_team() {
        let to_bench = run(&[
            leave("l", "2025-11-18", "T1"),
            with(join("j", "2025-11-18", "T1"), |e| e.is_sub = true),
        ]);
        assert_eq!(kinds(&to_bench), [(Kind::Bench, Some("T1"), Some("T1"))]);

        let to_starter = run(&[
            with(leave("l", "2025-11-18", "T1"), |e| e.is_sub = true),
            join("j", "2025-11-18", "T1"),
        ]);
        assert_eq!(kinds(&to_starter), [(Kind::Sub, Some("T1"), Some("T1"))]);
    }

    fn loan(mut event: RawEvent, preload: &str, status: Option<&str>) -> RawEvent {
        event.preload = preload.to_owned();
        event.status = status.map(str::to_owned);
        event.is_loan = true;
        event
    }

    #[test]
    fn a_loan_goes_from_the_lending_team_to_the_borrowing_team() {
        let transfers = run(&[
            loan(leave("ol", "2025-11-18", "T1"), "loaned_to", None),
            loan(
                join("oj", "2025-11-18", "T1"),
                "loaned_to",
                Some("loaned_out"),
            ),
            loan(
                join("dj", "2025-11-18", "BNK"),
                "loaned_from",
                Some("on_loan"),
            ),
        ]);
        assert_eq!(kinds(&transfers), [(Kind::Loan, Some("T1"), Some("BNK"))]);
        assert_eq!(transfers[0].join_event.as_deref(), Some("dj"));
    }

    #[test]
    fn a_loan_return_goes_back_to_the_lending_team() {
        let transfers = run(&[
            loan(
                leave("bl", "2026-06-01", "BNK"),
                "loan_end",
                Some("on_loan"),
            ),
            loan(
                leave("ol", "2026-06-01", "T1"),
                "loan_return",
                Some("loaned_out"),
            ),
            loan(join("oj", "2026-06-01", "T1"), "loan_return", None),
        ]);
        assert_eq!(kinds(&transfers), [(Kind::Loan, Some("BNK"), Some("T1"))]);
        assert_eq!(transfers[0].leave_event.as_deref(), Some("bl"));
        assert_eq!(transfers[0].join_event.as_deref(), Some("oj"));
    }

    #[test]
    fn academy_to_main_roster_is_a_promotion() {
        let transfers = run(&[
            with(leave("l", "2025-11-18", "T1 Academy"), |e| {
                e.preload = "to_main".to_owned();
            }),
            with(join("j", "2025-11-18", "T1"), |e| {
                e.preload = "from_academy".to_owned();
            }),
        ]);
        assert_eq!(
            kinds(&transfers),
            [(Kind::Promotion, Some("T1 Academy"), Some("T1"))]
        );
    }

    #[test]
    fn a_renamed_team_counts_as_the_same_team() {
        let renames = HashMap::from([("Old Name".to_owned(), "New Name".to_owned())]);
        let transfers = pair(
            &[
                leave("l", "2025-11-18", "Old Name"),
                join("j", "2025-12-01", "New Name"),
            ],
            &renames,
        );
        assert_eq!(transfers, []);
    }

    #[test]
    fn follows_rename_chains_without_looping_on_cycles() {
        let renames = HashMap::from([
            ("A".to_owned(), "B".to_owned()),
            ("B".to_owned(), "C".to_owned()),
            ("X".to_owned(), "Y".to_owned()),
            ("Y".to_owned(), "X".to_owned()),
        ]);
        let same = pair(
            &[leave("l", "2025-11-18", "A"), join("j", "2025-12-01", "C")],
            &renames,
        );
        assert_eq!(same, []);
        let cycle = pair(
            &[leave("l", "2025-11-18", "X"), join("j", "2025-12-01", "Z")],
            &renames,
        );
        assert_eq!(kinds(&cycle), [(Kind::Transfer, Some("X"), Some("Z"))]);
    }

    #[test]
    fn pairs_each_player_independently() {
        let other = |e: &mut RawEvent| e.player = "Chovy".to_owned();
        let transfers = run(&[
            leave("fl", "2025-11-18", "T1"),
            with(leave("cl", "2025-11-19", "Gen.G"), other),
            with(join("cj", "2025-11-20", "T1"), other),
            join("fj", "2025-11-21", "Gen.G"),
        ]);
        let pairs: Vec<_> = transfers
            .iter()
            .map(|t| {
                (
                    t.player.as_str(),
                    t.leave_event.as_deref(),
                    t.join_event.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            pairs,
            [
                ("Chovy", Some("cl"), Some("cj")),
                ("Faker", Some("fl"), Some("fj")),
            ]
        );
    }
}
