use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde_json::Value;

use crate::leaguepedia::Row;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Join,
    Leave,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Join => "join",
            Self::Leave => "leave",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RosterChange {
    pub id: String,
    pub date: NaiveDate,
    pub player: String,
    pub team: String,
    pub direction: Direction,
    pub roles_ingame: Vec<String>,
    pub role_modifier: Option<String>,
    pub status: Option<String>,
    pub preload: String,
    pub tags: Vec<String>,
    pub is_gcd: bool,
    pub news_id: Option<String>,
    pub line_in_news: Option<i32>,
    pub source: Option<String>,
}

impl TryFrom<&Row> for RosterChange {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id = text(row, "RosterChangeId")?.to_owned();
        let parse = || -> Result<Self> {
            Ok(Self {
                date: date(row, "Date")?,
                player: required(row, "Player")?,
                team: required(row, "Team")?,
                direction: match text(row, "Direction")? {
                    "Join" => Direction::Join,
                    "Leave" => Direction::Leave,
                    other => bail!("unknown direction `{other}`"),
                },
                roles_ingame: list(row, "RolesIngame", ';'),
                role_modifier: optional(row, "RoleModifier"),
                status: optional(row, "Status"),
                preload: text(row, "Preload")?.to_owned(),
                tags: list(row, "Tags", ','),
                is_gcd: flag(row, "IsGCD"),
                news_id: optional(row, "NewsId"),
                line_in_news: optional(row, "LineInNews")
                    .map(|line| line.parse())
                    .transpose()
                    .context("invalid `LineInNews`")?,
                source: optional(row, "Source"),
                id: id.clone(),
            })
        };
        parse().with_context(|| format!("roster change `{id}`"))
    }
}

#[derive(Debug, Clone)]
pub struct Tournament {
    pub page: String,
    pub league: String,
    pub date_start: NaiveDate,
    pub date_end: Option<NaiveDate>,
}

impl TryFrom<&Row> for Tournament {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        Ok(Self {
            page: required(row, "OverviewPage")?,
            league: text(row, "League")?.to_owned(),
            date_start: date(row, "DateStart")?,
            date_end: optional(row, "Date")
                .map(|_| date(row, "Date"))
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TournamentRoster {
    pub tournament: String,
    pub team: String,
}

impl TryFrom<&Row> for TournamentRoster {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        Ok(Self {
            tournament: required(row, "OverviewPage")?,
            team: required(row, "Team")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Redirect {
    pub alias: String,
    pub page: String,
}

impl Redirect {
    pub fn team(row: &Row) -> Result<Self> {
        Ok(Self {
            alias: required(row, "AllName")?,
            page: required(row, "Page")?,
        })
    }

    pub fn player(row: &Row) -> Result<Self> {
        Ok(Self {
            alias: required(row, "AllName")?,
            page: required(row, "OverviewPage")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TeamInfo {
    pub page: String,
    pub short: Option<String>,
    pub region: Option<String>,
    pub org: Option<String>,
    pub image: Option<String>,
}

impl TryFrom<&Row> for TeamInfo {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        Ok(Self {
            page: required(row, "Page")?,
            short: optional(row, "Short"),
            region: optional(row, "Region"),
            org: optional(row, "OrganizationPage"),
            image: optional(row, "Image"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub page: String,
    pub real_name: Option<String>,
    pub country: Option<String>,
    pub role: Option<String>,
    pub image: Option<String>,
}

impl TryFrom<&Row> for PlayerInfo {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        Ok(Self {
            page: required(row, "OverviewPage")?,
            real_name: optional(row, "Name"),
            country: optional(row, "Country"),
            role: optional(row, "Role"),
            image: optional(row, "Image"),
        })
    }
}

fn text<'a>(row: &'a Row, key: &str) -> Result<&'a str> {
    row.get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing field `{key}`"))
}

fn required(row: &Row, key: &str) -> Result<String> {
    optional(row, key).with_context(|| format!("empty field `{key}`"))
}

fn optional(row: &Row, key: &str) -> Option<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn flag(row: &Row, key: &str) -> bool {
    row.get(key).and_then(Value::as_str) == Some("1")
}

fn list(row: &Row, key: &str, delimiter: char) -> Vec<String> {
    optional(row, key)
        .map(|value| {
            value
                .split(delimiter)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn date(row: &Row, key: &str) -> Result<NaiveDate> {
    let value = text(row, key)?;
    let day = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .with_context(|| format!("invalid date `{value}` in `{key}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(value: Value) -> Row {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn parses_a_roster_change_row() {
        let change = RosterChange::try_from(&row(json!({
            "RosterChangeId": "2026-11-18_Ucal_DN SOOPers_0",
            "Date": "2026-11-18 00:00:00",
            "Player": "Ucal",
            "Team": "DN SOOPers",
            "Direction": "Join",
            "RolesIngame": "Mid",
            "RoleModifier": "",
            "Status": "",
            "Preload": "join",
            "Tags": "",
            "IsGCD": "0",
            "NewsId": "News/2026-11-18_4",
            "LineInNews": "1",
            "Source": "https://x.com/SOOPers_LoL/status/1;;;X Post;;;x.com;;;;;;ref;;;;;;:::",
        })))
        .unwrap();
        assert_eq!(change.date, NaiveDate::from_ymd_opt(2026, 11, 18).unwrap());
        assert_eq!(change.direction, Direction::Join);
        assert_eq!(change.roles_ingame, ["Mid"]);
        assert_eq!(change.role_modifier, None);
        assert_eq!(change.line_in_news, Some(1));
        assert!(!change.is_gcd);
    }

    #[test]
    fn rejects_unknown_direction_with_the_row_id() {
        let error = RosterChange::try_from(&row(json!({
            "RosterChangeId": "x_1",
            "Date": "2026-11-18 00:00:00",
            "Player": "Ucal",
            "Team": "DN SOOPers",
            "Direction": "Sideways",
            "Preload": "join",
        })))
        .unwrap_err();
        assert!(format!("{error:#}").contains("x_1"));
    }
}
