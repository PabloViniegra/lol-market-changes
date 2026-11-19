use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use chrono::{Days, NaiveDate};
use serde_json::{Value, json};

use crate::leaguepedia::{Client, Query, Row, in_list};
use crate::model::TournamentRoster;
use crate::normalize::{LEAGUE_NAMES, Snapshot, page_title};

// Long enough to include the previous season's rosters, so off-season moves stay in scope.
const TOURNAMENT_LOOKBACK_DAYS: u64 = 365;
// Older photos are not worth the extra pages: a player active in the window has a recent one.
const PLAYER_IMAGE_LOOKBACK_DAYS: u64 = 730;
// Queries are POSTed, so the ~8 KB URL limit of GET does not apply; a 69 KB `where` was
// accepted in testing. This keeps two IN lists well below that.
const MAX_IN_CHARS: usize = 20_000;
const QUOTES_AND_COMMA: usize = 3;
const ROSTER_CHANGE_FIELDS: &str = "Date_Sort=Date,Player,Direction,Team,Roles,RolesIngame,\
    RolesStaff,RoleDisplay,Role,RoleModifier,Status,CurrentTeamPriority,PlayerUnlinked,\
    AlreadyJoined,Tournaments,Source,IsGCD,Preload,PreloadSortNumber,Tags,NewsId,RosterChangeId,\
    N_LineInNews=LineInNews";

/// Raw `RosterChanges.Team` and `.Player` values the snapshot queried, i.e. the rows it re-fetched.
#[derive(Debug, Default)]
pub struct Scope {
    pub teams: Vec<String>,
    pub players: Vec<String>,
}

/// The tournaments, rosters and team redirects that decide which teams are in scope. They change
/// rarely, so runs reuse them for a day (`db::load_scope_rows`).
#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRows {
    pub tournaments: Vec<Row>,
    pub tournament_rosters: Vec<Row>,
    pub team_redirects: Vec<Row>,
}

impl ScopeRows {
    pub fn to_json(&self) -> Value {
        json!({
            "tournaments": self.tournaments,
            "tournament_rosters": self.tournament_rosters,
            "team_redirects": self.team_redirects,
        })
    }

    pub fn from_json(value: Value) -> Result<Self> {
        let rows = |key: &str| -> Result<Vec<Row>> {
            Ok(serde_json::from_value(
                value
                    .get(key)
                    .cloned()
                    .with_context(|| format!("cached scope has no `{key}`"))?,
            )?)
        };
        Ok(Self {
            tournaments: rows("tournaments")?,
            tournament_rosters: rows("tournament_rosters")?,
            team_redirects: rows("team_redirects")?,
        })
    }
}

pub async fn fetch_scope(client: &mut Client, since: NaiveDate) -> Result<ScopeRows> {
    let tournaments = client
        .query_all(&Query {
            tables: "Tournaments".into(),
            fields: "Name,OverviewPage,League,Region,DateStart,Date,TournamentLevel".into(),
            where_clause: format!(
                "{} AND TournamentLevel=\"Primary\" AND DateStart>=\"{}\"",
                in_list("League", &LEAGUE_NAMES)?,
                since - Days::new(TOURNAMENT_LOOKBACK_DAYS)
            ),
            order_by: "DateStart".into(),
            ..Default::default()
        })
        .await?;

    let tournament_rosters =
        query_chunked(client, &distinct(&tournaments, "OverviewPage"), |chunk| {
            Ok(Query {
                tables: "TournamentRosters".into(),
                fields: "Team,OverviewPage,Region,Short,RosterLinks,Roles".into(),
                where_clause: in_list("OverviewPage", chunk)?,
                order_by: "OverviewPage,Team".into(),
                ..Default::default()
            })
        })
        .await?;

    let team_redirects =
        fetch_team_redirects(client, &distinct(&tournament_rosters, "Team"), true).await?;
    Ok(ScopeRows {
        tournaments,
        tournament_rosters,
        team_redirects,
    })
}

/// Fetches every row needed to normalise roster changes on or after `since`: the in-scope
/// teams' changes plus the full history of their players in the window, so moves from or to
/// teams outside the four leagues keep both ends.
pub async fn fetch_snapshot(
    client: &mut Client,
    since: NaiveDate,
    scope: ScopeRows,
) -> Result<(Snapshot, Scope)> {
    let ScopeRows {
        tournaments,
        tournament_rosters,
        mut team_redirects,
    } = scope;
    let scope_names = union([
        distinct(&tournament_rosters, "Team"),
        distinct(&team_redirects, "Page"),
        distinct(&team_redirects, "AllName"),
    ]);
    let scope_changes = query_chunked(client, &scope_names, |chunk| {
        roster_changes(since, &in_list("Team", chunk)?)
    })
    .await?;
    let scope_players = distinct(&scope_changes, "Player");
    let player_changes = query_chunked(client, &scope_players, |chunk| {
        roster_changes(since, &in_list("Player", chunk)?)
    })
    .await?;
    let roster_changes = merge_by_id(scope_changes, player_changes);

    let known: BTreeSet<String> = distinct(&team_redirects, "AllName").into_iter().collect();
    let unknown_teams: Vec<String> = distinct(&roster_changes, "Team")
        .into_iter()
        .filter(|team| !known.contains(team))
        .collect();
    team_redirects.extend(fetch_team_redirects(client, &unknown_teams, false).await?);

    let team_pages = union([distinct(&team_redirects, "Page"), unknown_teams]);
    let teams = query_chunked(client, &team_pages, |chunk| {
        Ok(Query {
            tables: "Teams".into(),
            fields: "_pageName=Page,Name,Short,Region,OrganizationPage,IsDisbanded,RenamedTo"
                .into(),
            where_clause: in_list("_pageName", chunk)?,
            order_by: "_pageName".into(),
            ..Default::default()
        })
    })
    .await?;

    let roster_links = tournament_rosters
        .iter()
        .map(TournamentRoster::try_from)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|roster| roster.players.into_iter().map(|(player, _)| player));
    let player_titles = union([distinct(&roster_changes, "Player")
        .into_iter()
        .chain(roster_links)
        .map(|name| page_title(&name))
        .collect()]);
    let player_redirects = query_chunked(client, &player_titles, |chunk| {
        Ok(Query {
            tables: "PlayerRedirects".into(),
            fields: "AllName,OverviewPage,ID".into(),
            where_clause: in_list("AllName", chunk)?,
            order_by: "OverviewPage,AllName".into(),
            ..Default::default()
        })
    })
    .await?;

    let players = query_chunked(
        client,
        &distinct(&player_redirects, "OverviewPage"),
        |chunk| {
            Ok(Query {
                tables: "Players".into(),
                fields: "ID,OverviewPage,Player,Name,Country,Residency,Role,Team,Image,IsRetired,\
                     IsSubstitute,IsTrainee,IsPersonality"
                    .into(),
                where_clause: in_list("OverviewPage", chunk)?,
                order_by: "OverviewPage".into(),
                ..Default::default()
            })
        },
    )
    .await?;

    let player_images = query_chunked(
        client,
        &distinct(&player_redirects, "OverviewPage"),
        |chunk| {
            Ok(Query {
                tables: "PlayerImages=PI,Tournaments=T".into(),
                fields: "PI.Link=Link,PI.FileName=FileName,T.DateStart=DateStart".into(),
                join_on: "PI.Tournament=T.OverviewPage".into(),
                where_clause: format!(
                    "{} AND T.DateStart>=\"{}\"",
                    in_list("PI.Link", chunk)?,
                    since - Days::new(PLAYER_IMAGE_LOOKBACK_DAYS)
                ),
                order_by: "PI.Link,PI.FileName".into(),
            })
        },
    )
    .await?;

    let snapshot = Snapshot {
        tournaments,
        tournament_rosters,
        team_redirects,
        teams,
        roster_changes,
        player_redirects,
        players,
        player_images,
    };
    let scope = Scope {
        teams: scope_names,
        players: scope_players,
    };
    Ok((snapshot, scope))
}

async fn fetch_team_redirects(
    client: &mut Client,
    teams: &[String],
    by_page: bool,
) -> Result<Vec<Row>> {
    query_chunked(client, teams, |chunk| {
        let where_clause = if by_page {
            format!(
                "{} OR {}",
                in_list("_pageName", chunk)?,
                in_list("AllName", chunk)?
            )
        } else {
            in_list("AllName", chunk)?
        };
        Ok(Query {
            tables: "TeamRedirects".into(),
            fields: "_pageName=Page,AllName".into(),
            where_clause,
            order_by: "_pageName,AllName".into(),
            ..Default::default()
        })
    })
    .await
}

fn roster_changes(since: NaiveDate, filter: &str) -> Result<Query> {
    Ok(Query {
        tables: "RosterChanges".into(),
        fields: ROSTER_CHANGE_FIELDS.into(),
        where_clause: format!("Date_Sort>=\"{since}\" AND {filter}"),
        order_by: "Date_Sort,NewsId,N_LineInNews".into(),
        ..Default::default()
    })
}

pub async fn query_chunked(
    client: &mut Client,
    values: &[String],
    build: impl Fn(&[String]) -> Result<Query>,
) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for chunk in chunk_by_chars(values, MAX_IN_CHARS) {
        rows.extend(client.query_all(&build(chunk)?).await?);
    }
    Ok(rows)
}

fn chunk_by_chars(values: &[String], max: usize) -> Vec<&[String]> {
    let mut chunks = Vec::new();
    let (mut start, mut size) = (0, 0);
    for (i, value) in values.iter().enumerate() {
        let len = value.len() + QUOTES_AND_COMMA;
        if i > start && size + len > max {
            chunks.push(&values[start..i]);
            (start, size) = (i, 0);
        }
        size += len;
    }
    if start < values.len() {
        chunks.push(&values[start..]);
    }
    chunks
}

pub fn distinct(rows: &[Row], field: &str) -> Vec<String> {
    union([rows
        .iter()
        .filter_map(|row| row.get(field).and_then(Value::as_str))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()])
}

fn union<const N: usize>(lists: [Vec<String>; N]) -> Vec<String> {
    lists
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn merge_by_id(first: Vec<Row>, second: Vec<Row>) -> Vec<Row> {
    let mut by_id: BTreeMap<String, Row> = BTreeMap::new();
    for row in first.into_iter().chain(second) {
        let id = row
            .get("RosterChangeId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        by_id.insert(id, row);
    }
    by_id.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(id: &str, player: &str) -> Row {
        json!({"RosterChangeId": id, "Player": player})
            .as_object()
            .unwrap()
            .clone()
    }

    #[test]
    fn merge_by_id_drops_duplicates() {
        let merged = merge_by_id(
            vec![row("a", "x"), row("b", "y")],
            vec![row("b", "y"), row("c", "z")],
        );
        assert_eq!(distinct(&merged, "RosterChangeId"), ["a", "b", "c"]);
    }

    #[test]
    fn chunk_by_chars_respects_the_budget_and_keeps_every_value() {
        let values: Vec<String> = ["aaaa", "bb", "cccccccccc", "d"].map(String::from).into();
        let chunks = chunk_by_chars(&values, 12);
        assert_eq!(chunks, [&values[..2], &values[2..3], &values[3..]]);
        assert!(chunk_by_chars(&[], 10).is_empty());
    }

    #[test]
    fn distinct_skips_empty_values_and_sorts() {
        let rows = vec![row("1", "b"), row("2", ""), row("3", "a"), row("4", "b")];
        assert_eq!(distinct(&rows, "Player"), ["a", "b"]);
    }
}
