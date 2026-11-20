use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Days, NaiveDate, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::pairing::{Kind, lineage};

pub const RUMOR_MATCH_LEAD_DAYS: u64 = 7;
pub const RUMOR_MATCH_WINDOW_DAYS: u64 = 90;
pub const RUMOR_EXPIRY_DAYS: u64 = 60;

/// The latest calendar date anywhere on Earth (UTC+14). A rumor is dated where it was
/// reported, so a Korean report from this morning can be a day ahead of UTC.
pub fn latest_report_date(now: DateTime<Utc>) -> NaiveDate {
    (now + TimeDelta::hours(14)).date_naive()
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rumor {
    pub id: String,
    pub player: String,
    pub from_team: Option<String>,
    pub to_team: Option<String>,
    pub role: Option<String>,
    pub kind: Option<String>,
    pub reported_at: NaiveDate,
    pub sources: Vec<Source>,
    pub reliability: i16,
    pub status: Status,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub url: String,
    pub outlet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Rumor,
    Confirmed,
    Denied,
    Stale,
    Contradicted,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rumor => "rumor",
            Self::Confirmed => "confirmed",
            Self::Denied => "denied",
            Self::Stale => "stale",
            Self::Contradicted => "contradicted",
        }
    }
}

/// An official transfer a rumor can be confirmed or contradicted by.
#[derive(Debug, Clone, PartialEq)]
pub struct Official {
    pub id: String,
    pub player: String,
    pub from_team: Option<String>,
    pub to_team: Option<String>,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub status: Status,
    pub transfer_id: Option<String>,
}

/// Parses `rumors.yaml` and checks the rules of §4.2, reporting every problem at once.
pub fn parse(yaml: &str, latest: NaiveDate) -> Result<Vec<Rumor>> {
    let rumors: Vec<Rumor> = yaml_serde::from_str::<Option<Vec<Rumor>>>(yaml)
        .context("rumors.yaml does not match the schema")?
        .unwrap_or_default();
    let mut problems = Vec::new();
    let mut ids = HashSet::new();
    for rumor in &rumors {
        let mut problem = |message: String| problems.push(format!("{}: {message}", rumor.id));
        if !ids.insert(rumor.id.as_str()) {
            problem("duplicate id".into());
        }
        if rumor.player.trim().is_empty() {
            problem("player is required".into());
        }
        if rumor.from_team.is_none() && rumor.to_team.is_none() {
            problem("from_team or to_team is required".into());
        }
        if rumor.reported_at > latest {
            problem(format!(
                "reported_at {} is in the future",
                rumor.reported_at
            ));
        }
        if rumor.sources.is_empty() {
            problem("at least one source is required".into());
        }
        if !(1..=5).contains(&rumor.reliability) {
            problem(format!(
                "reliability {} is not between 1 and 5",
                rumor.reliability
            ));
        }
        if !matches!(rumor.status, Status::Rumor | Status::Denied) {
            problem(format!(
                "status {} is set by the reconciler",
                rumor.status.as_str()
            ));
        }
        if let Some(kind) = &rumor.kind
            && !Kind::ALL.iter().any(|k| k.as_str() == kind)
        {
            problem(format!("unknown kind {kind}"));
        }
    }
    if !problems.is_empty() {
        bail!("invalid rumors.yaml:\n{}", problems.join("\n"));
    }
    Ok(rumors)
}

/// Computes the status of `rumor` from the official transfers (§6.4). `renames` lets a rumor
/// name either page of a renamed team.
pub fn reconcile(
    rumor: &Rumor,
    officials: &[Official],
    renames: &HashMap<String, String>,
    today: NaiveDate,
) -> Outcome {
    let plain = |status| Outcome {
        status,
        transfer_id: None,
    };
    if rumor.status == Status::Denied {
        return plain(Status::Denied);
    }
    let key = |team: &str| lineage(renames, team).to_lowercase();
    let same = |team: &Option<String>, wanted: &str| {
        team.as_deref().is_some_and(|team| key(team) == key(wanted))
    };
    let player = rumor.player.to_lowercase();
    let window = (rumor.reported_at - Days::new(RUMOR_MATCH_LEAD_DAYS))
        ..=(rumor.reported_at + Days::new(RUMOR_MATCH_WINDOW_DAYS));
    let in_window: Vec<&Official> = officials
        .iter()
        .filter(|o| o.player.to_lowercase() == player && window.contains(&o.date))
        .collect();
    let matched = in_window
        .iter()
        .filter(|o| match (&rumor.to_team, &rumor.from_team) {
            (Some(to), _) => same(&o.to_team, to),
            (None, Some(from)) => same(&o.from_team, from),
            (None, None) => false,
        })
        .min_by_key(|o| o.date);
    // Only a move to a third team contradicts: a departure rumor gives no destination to
    // contradict, leaving for free agency does not rule out the rumored team, and joining the
    // origin (a loan return, a promotion) can precede the rumored move.
    let contradicted = rumor.to_team.as_deref().is_some_and(|to| {
        in_window.iter().any(|o| {
            o.to_team.is_some()
                && !same(&o.to_team, to)
                && !rumor
                    .from_team
                    .as_deref()
                    .is_some_and(|from| same(&o.to_team, from))
        })
    });
    match matched {
        Some(official) => Outcome {
            status: Status::Confirmed,
            transfer_id: Some(official.id.clone()),
        },
        None if contradicted => plain(Status::Contradicted),
        None if today > rumor.reported_at + Days::new(RUMOR_EXPIRY_DAYS) => plain(Status::Stale),
        None => plain(Status::Rumor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    const TODAY: &str = "2026-11-18";

    const VALID: &str = "
- id: 2026-11-10-caps-g2-to-fnc
  player: Caps
  from_team: G2 Esports
  to_team: Fnatic
  role: Mid
  kind: transfer
  reported_at: 2026-11-10
  sources:
    - url: https://example.com/article
      outlet: Sheep Esports
  reliability: 4
  status: rumor
  notes: Optional free text
- id: 2026-11-12-faker-retires
  player: Faker
  from_team: T1
  reported_at: 2026-11-12
  sources:
    - url: https://example.com/other
  reliability: 2
  status: denied
";

    fn rumor() -> Rumor {
        parse(VALID, date(TODAY)).unwrap().remove(0)
    }

    fn errors(yaml: &str) -> String {
        format!("{:#}", parse(yaml, date(TODAY)).unwrap_err())
    }

    fn official(id: &str, from: Option<&str>, to: Option<&str>, on: &str) -> Official {
        Official {
            id: id.to_owned(),
            player: "Caps".to_owned(),
            from_team: from.map(str::to_owned),
            to_team: to.map(str::to_owned),
            date: date(on),
        }
    }

    fn outcome(rumor: &Rumor, officials: &[Official], today: &str) -> Outcome {
        reconcile(rumor, officials, &HashMap::new(), date(today))
    }

    fn confirmed(id: &str) -> Outcome {
        Outcome {
            status: Status::Confirmed,
            transfer_id: Some(id.to_owned()),
        }
    }

    fn plain(status: Status) -> Outcome {
        Outcome {
            status,
            transfer_id: None,
        }
    }

    #[test]
    fn parses_the_documented_shape() {
        let rumors = parse(VALID, date(TODAY)).unwrap();
        assert_eq!(rumors.len(), 2);
        assert_eq!(rumors[0].to_team.as_deref(), Some("Fnatic"));
        assert_eq!(
            rumors[0].sources[0].outlet.as_deref(),
            Some("Sheep Esports")
        );
        assert_eq!(rumors[1].to_team, None);
        assert_eq!(rumors[1].status, Status::Denied);
    }

    #[test]
    fn an_empty_file_has_no_rumors() {
        assert_eq!(parse("", date(TODAY)).unwrap(), []);
        assert_eq!(parse("[]", date(TODAY)).unwrap(), []);
    }

    #[test]
    fn the_latest_report_date_is_today_in_utc_plus_14() {
        let at = |instant: &str| latest_report_date(instant.parse().unwrap());
        assert_eq!(at("2026-11-18T09:59:59Z"), date("2026-11-18"));
        assert_eq!(at("2026-11-18T10:00:00Z"), date("2026-11-19"));
    }

    #[test]
    fn reports_every_broken_rule() {
        let yaml = "
- id: dup
  player: ''
  reported_at: 2026-11-19
  sources: []
  reliability: 6
  status: confirmed
  kind: trade
- id: dup
  player: Caps
  to_team: Fnatic
  reported_at: 2026-11-01
  sources: [{url: https://example.com}]
  reliability: 0
  status: stale
- id: auto
  player: Caps
  to_team: Fnatic
  reported_at: 2026-11-01
  sources: [{url: https://example.com}]
  reliability: 3
  status: contradicted
";
        let message = errors(yaml);
        for expected in [
            "dup: duplicate id",
            "dup: player is required",
            "dup: from_team or to_team is required",
            "dup: reported_at 2026-11-19 is in the future",
            "dup: at least one source is required",
            "dup: reliability 6 is not between 1 and 5",
            "dup: status confirmed is set by the reconciler",
            "dup: unknown kind trade",
            "dup: reliability 0 is not between 1 and 5",
            "dup: status stale is set by the reconciler",
            "auto: status contradicted is set by the reconciler",
        ] {
            assert!(
                message.contains(expected),
                "missing {expected:?} in:\n{message}"
            );
        }
    }

    #[test]
    fn rejects_unknown_fields_and_bad_dates() {
        assert!(errors("- {id: a, player: Caps, to_team: T1, reported_at: 2026-11-01, sources: [{url: u}], reliability: 3, status: rumor, team: T1}").contains("team"));
        assert!(errors("- {id: a, player: Caps, to_team: T1, reported_at: yesterday, sources: [{url: u}], reliability: 3, status: rumor}").contains("reported_at"));
    }

    #[test]
    fn confirms_with_an_official_move_to_the_same_team() {
        let officials = [
            official(
                "other",
                Some("G2 Esports"),
                Some("Team Vitality"),
                "2026-11-20",
            ),
            official("match", Some("G2 Esports"), Some("Fnatic"), "2026-11-25"),
        ];
        assert_eq!(outcome(&rumor(), &officials, TODAY), confirmed("match"));
    }

    #[test]
    fn matches_names_case_insensitively_and_across_renames() {
        let mut rumor = rumor();
        rumor.player = "caps".to_owned();
        rumor.to_team = Some("Fnatic Old".to_owned());
        let renames = HashMap::from([("Fnatic Old".to_owned(), "Fnatic".to_owned())]);
        let officials = [official("match", None, Some("fnatic"), "2026-11-25")];
        assert_eq!(
            reconcile(&rumor, &officials, &renames, date(TODAY)),
            confirmed("match")
        );
    }

    #[test]
    fn a_rumor_without_destination_is_confirmed_by_leaving_the_origin() {
        let mut rumor = rumor();
        rumor.to_team = None;
        let officials = [official("release", Some("G2 Esports"), None, "2026-11-20")];
        assert_eq!(outcome(&rumor, &officials, TODAY), confirmed("release"));
    }

    #[test]
    fn the_match_window_runs_from_a_week_before_to_ninety_days_after() {
        let rumor = rumor();
        let at = |on: &str| {
            outcome(
                &rumor,
                &[official("t", None, Some("Fnatic"), on)],
                "2027-03-01",
            )
        };
        assert_eq!(at("2026-11-03"), confirmed("t"));
        assert_eq!(at("2027-02-08"), confirmed("t"));
        assert_eq!(at("2026-11-02").status, Status::Stale);
        assert_eq!(at("2027-02-09").status, Status::Stale);
    }

    #[test]
    fn the_earliest_match_wins() {
        let officials = [
            official("loan", None, Some("Fnatic"), "2026-12-20"),
            official("first", None, Some("Fnatic"), "2026-11-20"),
        ];
        assert_eq!(outcome(&rumor(), &officials, TODAY), confirmed("first"));
    }

    #[test]
    fn a_move_to_another_team_contradicts() {
        let officials = [official(
            "elsewhere",
            Some("G2 Esports"),
            Some("Team Vitality"),
            "2026-11-20",
        )];
        assert_eq!(
            outcome(&rumor(), &officials, TODAY),
            plain(Status::Contradicted)
        );
    }

    #[test]
    fn a_match_in_the_window_beats_a_contradiction() {
        let officials = [
            official("loan", None, Some("Team Vitality"), "2026-11-20"),
            official("match", None, Some("Fnatic"), "2027-01-20"),
        ];
        assert_eq!(
            outcome(&rumor(), &officials, "2027-01-21"),
            confirmed("match")
        );
    }

    #[test]
    fn leaving_or_moving_outside_the_window_does_not_contradict() {
        let officials = [
            official("release", Some("G2 Esports"), None, "2026-11-20"),
            official("early", None, Some("Team Vitality"), "2026-11-02"),
            official("late", None, Some("Team Vitality"), "2027-02-09"),
        ];
        assert_eq!(outcome(&rumor(), &officials, TODAY), plain(Status::Rumor));
    }

    #[test]
    fn joining_the_origin_team_does_not_contradict() {
        let officials = [official(
            "loan-return",
            Some("Team BDS"),
            Some("g2 esports"),
            "2026-11-05",
        )];
        assert_eq!(outcome(&rumor(), &officials, TODAY), plain(Status::Rumor));
    }

    #[test]
    fn a_contradiction_outlasts_the_expiry() {
        let officials = [official(
            "elsewhere",
            None,
            Some("Team Vitality"),
            "2026-11-20",
        )];
        assert_eq!(
            outcome(&rumor(), &officials, "2027-01-10"),
            plain(Status::Contradicted)
        );
    }

    #[test]
    fn a_departure_rumor_is_never_contradicted() {
        let mut rumor = rumor();
        rumor.to_team = None;
        let officials = [official(
            "elsewhere",
            None,
            Some("Team Vitality"),
            "2026-11-20",
        )];
        assert_eq!(outcome(&rumor, &officials, TODAY), plain(Status::Rumor));
    }

    #[test]
    fn another_players_move_does_not_confirm() {
        let mut other = official("t", None, Some("Fnatic"), "2026-11-20");
        other.player = "Humanoid".to_owned();
        assert_eq!(outcome(&rumor(), &[other], TODAY), plain(Status::Rumor));
    }

    #[test]
    fn goes_stale_after_sixty_days_without_a_match() {
        let rumor = rumor();
        assert_eq!(outcome(&rumor, &[], "2027-01-09"), plain(Status::Rumor));
        assert_eq!(outcome(&rumor, &[], "2027-01-10"), plain(Status::Stale));
    }

    #[test]
    fn a_stale_rumor_is_confirmed_by_a_later_match_in_the_window() {
        let officials = [official("late", None, Some("Fnatic"), "2027-02-01")];
        assert_eq!(
            outcome(&rumor(), &officials, "2027-02-02"),
            confirmed("late")
        );
    }

    #[test]
    fn denied_always_wins() {
        let mut rumor = rumor();
        rumor.status = Status::Denied;
        let officials = [
            official("match", None, Some("Fnatic"), "2026-11-20"),
            official("elsewhere", None, Some("Team Vitality"), "2026-11-21"),
        ];
        assert_eq!(outcome(&rumor, &officials, TODAY), plain(Status::Denied));
    }
}
