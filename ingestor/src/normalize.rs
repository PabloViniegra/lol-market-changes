use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use chrono::NaiveDate;

use crate::leaguepedia::Row;
use crate::model::{
    Direction, PlayerInfo, Redirect, RosterChange, TeamInfo, Tournament, TournamentRoster,
};

pub const FILE_URL: &str = "https://lol.fandom.com/wiki/Special:FilePath/";

/// Raw Cargo rows as fetched from Leaguepedia; one field per table.
#[derive(Debug, Default)]
pub struct Snapshot {
    pub tournaments: Vec<Row>,
    pub tournament_rosters: Vec<Row>,
    pub team_redirects: Vec<Row>,
    pub teams: Vec<Row>,
    pub roster_changes: Vec<Row>,
    pub player_redirects: Vec<Row>,
    pub players: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Team {
    pub name: String,
    pub short: Option<String>,
    pub league: String,
    pub region: Option<String>,
    pub org: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamSeason {
    pub team: String,
    pub league: String,
    pub tournament: String,
    pub date_start: NaiveDate,
    pub date_end: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub name: String,
    pub real_name: Option<String>,
    pub country: Option<String>,
    pub role: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawEvent {
    pub id: String,
    pub date: NaiveDate,
    pub player: String,
    pub player_raw: String,
    pub team: String,
    pub team_raw: String,
    pub direction: Direction,
    pub role: Option<String>,
    pub preload: String,
    pub status: Option<String>,
    pub is_sub: bool,
    pub is_loan: bool,
    pub is_retired: bool,
    pub news_id: Option<String>,
    pub line_in_news: Option<i32>,
    pub source_url: Option<String>,
}

#[derive(Debug, Default)]
pub struct Normalized {
    pub teams: Vec<Team>,
    pub team_seasons: Vec<TeamSeason>,
    pub players: Vec<Player>,
    pub raw_events: Vec<RawEvent>,
    pub skipped: Skipped,
}

#[derive(Debug, Default, PartialEq)]
pub struct Skipped {
    pub staff: usize,
    pub game_changers: usize,
    pub team_renames: usize,
    pub out_of_scope: usize,
}

/// Leaguepedia `Tournaments.League` → league code. LTA North was the North American top
/// league in 2025, between LCS seasons, so it counts as LCS.
pub fn league_code(league: &str) -> Option<&'static str> {
    match league {
        "LoL Champions Korea" => Some("LCK"),
        "Tencent LoL Pro League" => Some("LPL"),
        "LoL EMEA Championship" => Some("LEC"),
        "League of Legends Championship Series"
        | "League of Legends Championship of The Americas North" => Some("LCS"),
        _ => None,
    }
}

pub const LEAGUE_NAMES: [&str; 5] = [
    "LoL Champions Korea",
    "Tencent LoL Pro League",
    "LoL EMEA Championship",
    "League of Legends Championship Series",
    "League of Legends Championship of The Americas North",
];

/// MediaWiki stores page titles with an uppercase first letter, so `kyeahoo` lives at `Kyeahoo`.
pub fn page_title(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

pub fn normalize(snapshot: &Snapshot) -> Result<Normalized> {
    let tournaments = parse::<Tournament>(&snapshot.tournaments)?;
    let rosters = parse::<TournamentRoster>(&snapshot.tournament_rosters)?;
    let changes = parse::<RosterChange>(&snapshot.roster_changes)?;
    let team_infos = parse::<TeamInfo>(&snapshot.teams)?;
    let player_infos = parse::<PlayerInfo>(&snapshot.players)?;
    let team_aliases = aliases(&snapshot.team_redirects, Redirect::team)?;
    let player_aliases = aliases(&snapshot.player_redirects, Redirect::player)?;

    let resolve_team = |raw: &str| resolve(&team_aliases, raw);
    let resolve_player = |raw: &str| resolve(&player_aliases, raw);

    let leagues: HashMap<&str, (&Tournament, &'static str)> = tournaments
        .iter()
        .filter_map(|t| league_code(&t.league).map(|code| (t.page.as_str(), (t, code))))
        .collect();
    let team_seasons: Vec<TeamSeason> = rosters
        .iter()
        .filter_map(|roster| {
            let (tournament, league) = leagues.get(roster.tournament.as_str())?;
            let team = resolve_team(&roster.team);
            let season = TeamSeason {
                team: team.clone(),
                league: (*league).to_owned(),
                tournament: tournament.page.clone(),
                date_start: tournament.date_start,
                date_end: tournament.date_end,
            };
            Some(((team, tournament.page.clone()), season))
        })
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect();
    let scope_teams: BTreeSet<&str> = team_seasons.iter().map(|s| s.team.as_str()).collect();

    let mut skipped = Skipped::default();
    let mut events: Vec<RawEvent> = Vec::new();
    for change in &changes {
        if change.is_gcd {
            skipped.game_changers += 1;
        } else if change.tags.iter().any(|tag| tag == "TeamRename")
            || change.preload.starts_with("team_rename")
        {
            skipped.team_renames += 1;
        } else if change.roles_ingame.is_empty() {
            skipped.staff += 1;
        } else {
            events.push(raw_event(
                change,
                resolve_player(&change.player),
                resolve_team(&change.team),
            ));
        }
    }

    let players_in_scope: BTreeSet<String> = events
        .iter()
        .filter(|event| scope_teams.contains(event.team.as_str()))
        .map(|event| event.player.clone())
        .collect();
    let before = events.len();
    events.retain(|event| players_in_scope.contains(&event.player));
    skipped.out_of_scope = before - events.len();
    events.sort_by(|a, b| (a.date, &a.player, &a.id).cmp(&(b.date, &b.player, &b.id)));

    let latest_league: BTreeMap<&str, &str> = {
        let mut by_team: BTreeMap<&str, &TeamSeason> = BTreeMap::new();
        for season in &team_seasons {
            let entry = by_team.entry(&season.team).or_insert(season);
            if season.date_start > entry.date_start {
                *entry = season;
            }
        }
        by_team
            .into_iter()
            .map(|(team, s)| (team, s.league.as_str()))
            .collect()
    };
    let team_info: HashMap<&str, &TeamInfo> = team_infos
        .iter()
        .map(|info| (info.page.as_str(), info))
        .collect();
    let teams = events
        .iter()
        .map(|event| event.team.as_str())
        .chain(scope_teams.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|name| {
            let info = team_info.get(name);
            Team {
                name: name.to_owned(),
                short: info.and_then(|i| i.short.clone()),
                league: latest_league.get(name).unwrap_or(&"other").to_string(),
                region: info.and_then(|i| i.region.clone()),
                org: info.and_then(|i| i.org.clone()),
                logo_url: info.and_then(|i| i.image.as_deref()).map(file_url),
            }
        })
        .collect();

    let player_info: HashMap<&str, &PlayerInfo> = player_infos
        .iter()
        .map(|info| (info.page.as_str(), info))
        .collect();
    let players = players_in_scope
        .iter()
        .map(|name| {
            let info = player_info.get(name.as_str());
            Player {
                name: name.clone(),
                real_name: info.and_then(|i| i.real_name.clone()),
                country: info.and_then(|i| i.country.clone()),
                role: info.and_then(|i| i.role.clone()),
                image_url: info.and_then(|i| i.image.as_deref()).map(file_url),
            }
        })
        .collect();

    Ok(Normalized {
        teams,
        team_seasons,
        players,
        raw_events: events,
        skipped,
    })
}

fn raw_event(change: &RosterChange, player: String, team: String) -> RawEvent {
    let loan_status = matches!(change.status.as_deref(), Some("on_loan" | "loaned_out"));
    RawEvent {
        id: change.id.clone(),
        date: change.date,
        player,
        player_raw: change.player.clone(),
        team,
        team_raw: change.team.clone(),
        direction: change.direction,
        role: change.roles_ingame.first().cloned(),
        preload: change.preload.clone(),
        status: change.status.clone(),
        is_sub: change.role_modifier.as_deref() == Some("Sub"),
        is_loan: change.preload.contains("loan") || loan_status,
        is_retired: change.preload == "retirement",
        news_id: change.news_id.clone(),
        line_in_news: change.line_in_news,
        source_url: change.source.as_deref().and_then(first_source_url),
    }
}

/// `Source` holds entries separated by `:::`, each `url;;;title;;;site;;;...`.
fn first_source_url(source: &str) -> Option<String> {
    source
        .split(":::")
        .filter_map(|entry| entry.split(";;;").next())
        .map(str::trim)
        .find(|url| url.starts_with("http"))
        .map(str::to_owned)
}

fn file_url(file: &str) -> String {
    format!("{FILE_URL}{}", file.replace(' ', "_"))
}

fn parse<T>(rows: &[Row]) -> Result<Vec<T>>
where
    T: for<'a> TryFrom<&'a Row, Error = anyhow::Error>,
{
    rows.iter().map(T::try_from).collect()
}

fn aliases(rows: &[Row], parse: fn(&Row) -> Result<Redirect>) -> Result<HashMap<String, String>> {
    rows.iter()
        .map(|row| parse(row).map(|r| (r.alias, r.page)))
        .collect()
}

fn resolve(aliases: &HashMap<String, String>, raw: &str) -> String {
    aliases
        .get(raw)
        .or_else(|| aliases.get(&page_title(raw)))
        .cloned()
        .unwrap_or_else(|| page_title(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_title_uppercases_only_the_first_letter() {
        assert_eq!(page_title("kyeahoo"), "Kyeahoo");
        assert_eq!(page_title("Shiji mm"), "Shiji mm");
        assert_eq!(page_title(""), "");
    }

    #[test]
    fn first_source_url_takes_the_first_entry() {
        let source =
            "https://x.com/a/1;;;Post;;;x.com;;;;;;ref;;;;;;:::https://x.com/b/2;;;Post;;;x.com:::";
        assert_eq!(
            first_source_url(source).as_deref(),
            Some("https://x.com/a/1")
        );
        assert_eq!(first_source_url(";;;no url"), None);
    }

    #[test]
    fn maps_leaguepedia_league_names() {
        assert_eq!(league_code("LoL EMEA Championship"), Some("LEC"));
        assert_eq!(
            league_code("League of Legends Championship of The Americas North"),
            Some("LCS")
        );
        assert_eq!(
            league_code("League of Legends Championship of The Americas South"),
            None
        );
        assert!(LEAGUE_NAMES.iter().all(|name| league_code(name).is_some()));
    }
}
