use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};

use std::collections::{BTreeSet, HashMap};

use sqlx::Row;

use crate::fetch::ScopeRows;
use crate::model::Direction;
use crate::normalize::{Normalized, RawEvent};
use crate::pairing::{self, Transfer};
use crate::rumors::{self, Official, Rumor};

#[derive(Debug, Default, PartialEq)]
pub struct WriteStats {
    pub teams: u64,
    pub team_seasons: u64,
    pub players: u64,
    pub raw_events: u64,
    pub deleted_events: u64,
    pub transfers: u64,
}

/// Rows re-fetched by this run, by the raw names queried: events on or after `since` for these
/// players or teams that are missing from the snapshot were removed from the wiki.
pub struct Window<'a> {
    pub since: NaiveDate,
    pub players: &'a [String],
    pub teams: &'a [String],
}

pub async fn connect(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(url)
        .await
        .context("connecting to Postgres")?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("running migrations")?;
    Ok(pool)
}

pub async fn last_synced_date(pool: &PgPool) -> Result<Option<NaiveDate>> {
    let date = sqlx::query_scalar("select last_synced_date from sync_state where id = 1")
        .fetch_optional(pool)
        .await?;
    Ok(date.flatten())
}

pub async fn write(pool: &PgPool, data: &Normalized, window: &Window<'_>) -> Result<WriteStats> {
    let mut tx = pool.begin().await?;
    let stats = WriteStats {
        teams: upsert_teams(&mut tx, data).await?,
        team_seasons: upsert_team_seasons(&mut tx, data).await?,
        players: upsert_players(&mut tx, data).await?,
        ..WriteStats::default()
    };
    let raw_events = upsert_raw_events(&mut tx, data).await?;
    let deleted = mark_deleted(&mut tx, data, window).await?;
    let affected: Vec<String> = data
        .raw_events
        .iter()
        .map(|e| e.player.clone())
        .chain(deleted.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let stats = WriteStats {
        raw_events,
        deleted_events: deleted.len() as u64,
        transfers: rebuild_transfers(&mut tx, &affected).await?,
        ..stats
    };
    tx.commit().await?;
    Ok(stats)
}

/// Replaces the `rumors` table with `rumors`, each with its reconciled status (§6.4).
pub async fn sync_rumors(pool: &PgPool, rumors: &[Rumor], today: NaiveDate) -> Result<u64> {
    let mut tx = pool.begin().await?;
    let players: Vec<String> = rumors.iter().map(|r| r.player.to_lowercase()).collect();
    let officials: Vec<Official> = sqlx::query_as(
        "select id::text, player, from_team, to_team, date from transfers
         where lower(player) = any($1) and kind not in ('bench', 'sub')",
    )
    .bind(&players)
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|(id, player, from_team, to_team, date)| Official {
        id,
        player,
        from_team,
        to_team,
        date,
    })
    .collect();
    let renames = load_renames(&mut tx).await?;
    let outcomes: Vec<_> = rumors
        .iter()
        .map(|rumor| rumors::reconcile(rumor, &officials, &renames, today))
        .collect();
    let text = |f: fn(&Rumor) -> Option<String>| rumors.iter().map(f).collect::<Vec<_>>();
    let sources = rumors
        .iter()
        .map(|r| serde_json::to_string(&r.sources))
        .collect::<Result<Vec<_>, _>>()?;

    let removed = sqlx::query("delete from rumors where not (id = any($1))")
        .bind(text(|r| Some(r.id.clone())))
        .execute(&mut *tx)
        .await?;
    let upserted = sqlx::query(
        "insert into rumors (id, player, from_team, to_team, role, kind, reported_at, reliability,
           sources, notes, status, transfer_id)
         select id, player, from_team, to_team, role, kind, reported_at, reliability,
           sources::jsonb, notes, status, transfer_id::uuid
         from unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[],
           $7::date[], $8::int2[], $9::text[], $10::text[], $11::text[], $12::text[])
           as r(id, player, from_team, to_team, role, kind, reported_at, reliability, sources,
             notes, status, transfer_id)
         on conflict (id) do update set
           player = excluded.player, from_team = excluded.from_team, to_team = excluded.to_team,
           role = excluded.role, kind = excluded.kind, reported_at = excluded.reported_at,
           reliability = excluded.reliability, sources = excluded.sources,
           notes = excluded.notes, status = excluded.status, transfer_id = excluded.transfer_id,
           updated_at = now()
         where (rumors.player, rumors.from_team, rumors.to_team, rumors.role, rumors.kind,
             rumors.reported_at, rumors.reliability, rumors.sources, rumors.notes,
             rumors.status, rumors.transfer_id)
           is distinct from
           (excluded.player, excluded.from_team, excluded.to_team, excluded.role, excluded.kind,
             excluded.reported_at, excluded.reliability, excluded.sources, excluded.notes,
             excluded.status, excluded.transfer_id)",
    )
    .bind(text(|r| Some(r.id.clone())))
    .bind(text(|r| Some(r.player.clone())))
    .bind(text(|r| r.from_team.clone()))
    .bind(text(|r| r.to_team.clone()))
    .bind(text(|r| r.role.clone()))
    .bind(text(|r| r.kind.clone()))
    .bind(rumors.iter().map(|r| r.reported_at).collect::<Vec<_>>())
    .bind(rumors.iter().map(|r| r.reliability).collect::<Vec<_>>())
    .bind(sources)
    .bind(text(|r| r.notes.clone()))
    .bind(
        outcomes
            .iter()
            .map(|o| o.status.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        outcomes
            .iter()
            .map(|o| o.transfer_id.clone())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(removed.rows_affected() + upserted.rows_affected())
}

async fn load_renames(tx: &mut Transaction<'_, Postgres>) -> Result<HashMap<String, String>> {
    Ok(
        sqlx::query_as("select name, renamed_to from teams where renamed_to is not null")
            .fetch_all(&mut **tx)
            .await?
            .into_iter()
            .collect(),
    )
}

/// Scope rows fetched in the last 24 hours for a window that started on or before `since`.
pub async fn load_scope_rows(pool: &PgPool, since: NaiveDate) -> Result<Option<ScopeRows>> {
    let rows: Option<Value> = sqlx::query_scalar(
        "select rows from leaguepedia_cache
         where name = 'scope' and since <= $1 and fetched_at > now() - interval '24 hours'",
    )
    .bind(since)
    .fetch_optional(pool)
    .await?;
    rows.map(ScopeRows::from_json).transpose()
}

pub async fn save_scope_rows(pool: &PgPool, since: NaiveDate, rows: &ScopeRows) -> Result<()> {
    sqlx::query(
        "insert into leaguepedia_cache (name, since, rows) values ('scope', $1, $2)
         on conflict (name) do update set
           since = excluded.since, rows = excluded.rows, fetched_at = now()",
    )
    .bind(since)
    .bind(rows.to_json())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn record_run(
    pool: &PgPool,
    synced_date: Option<NaiveDate>,
    status: &str,
    stats: &Value,
) -> Result<()> {
    sqlx::query(
        "insert into sync_state (id, last_synced_date, last_run_at, last_run_status, stats)
         values (1, $1, now(), $2, $3)
         on conflict (id) do update set
           last_synced_date = coalesce(excluded.last_synced_date, sync_state.last_synced_date),
           last_run_at = excluded.last_run_at,
           last_run_status = excluded.last_run_status,
           stats = excluded.stats",
    )
    .bind(synced_date)
    .bind(status)
    .bind(stats)
    .execute(pool)
    .await?;
    Ok(())
}

// Every upsert only touches rows whose values changed, so a repeated run reports zero changes.

async fn upsert_teams(tx: &mut Transaction<'_, Postgres>, data: &Normalized) -> Result<u64> {
    let teams = &data.teams;
    let result = sqlx::query(
        "insert into teams (name, short, league, region, org, logo_url, renamed_to)
         select * from unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
           $6::text[], $7::text[])
         on conflict (name) do update set
           short = excluded.short, league = excluded.league, region = excluded.region,
           org = excluded.org, logo_url = excluded.logo_url, renamed_to = excluded.renamed_to
         where (teams.short, teams.league, teams.region, teams.org, teams.logo_url,
             teams.renamed_to)
           is distinct from
           (excluded.short, excluded.league, excluded.region, excluded.org, excluded.logo_url,
             excluded.renamed_to)",
    )
    .bind(teams.iter().map(|t| t.name.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.short.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.league.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.region.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.org.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.logo_url.clone()).collect::<Vec<_>>())
    .bind(
        teams
            .iter()
            .map(|t| t.renamed_to.clone())
            .collect::<Vec<_>>(),
    )
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

async fn upsert_team_seasons(tx: &mut Transaction<'_, Postgres>, data: &Normalized) -> Result<u64> {
    let seasons = &data.team_seasons;
    let result = sqlx::query(
        "insert into team_seasons (team, league, tournament, date_start, date_end, roster)
         select team, league, tournament, date_start, date_end, roster::jsonb
         from unnest($1::text[], $2::text[], $3::text[], $4::date[], $5::date[], $6::text[])
           as s(team, league, tournament, date_start, date_end, roster)
         on conflict (team, tournament) do update set
           league = excluded.league, date_start = excluded.date_start,
           date_end = excluded.date_end, roster = excluded.roster
         where (team_seasons.league, team_seasons.date_start, team_seasons.date_end,
                team_seasons.roster)
           is distinct from (excluded.league, excluded.date_start, excluded.date_end,
                             excluded.roster)",
    )
    .bind(seasons.iter().map(|s| s.team.clone()).collect::<Vec<_>>())
    .bind(seasons.iter().map(|s| s.league.clone()).collect::<Vec<_>>())
    .bind(
        seasons
            .iter()
            .map(|s| s.tournament.clone())
            .collect::<Vec<_>>(),
    )
    .bind(seasons.iter().map(|s| s.date_start).collect::<Vec<_>>())
    .bind(seasons.iter().map(|s| s.date_end).collect::<Vec<_>>())
    .bind(
        seasons
            .iter()
            .map(|s| serde_json::to_string(&s.roster))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

async fn upsert_players(tx: &mut Transaction<'_, Postgres>, data: &Normalized) -> Result<u64> {
    let players = &data.players;
    let result = sqlx::query(
        "insert into players (name, real_name, country, role, image_url)
         select * from unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[])
         on conflict (name) do update set
           real_name = excluded.real_name, country = excluded.country, role = excluded.role,
           image_url = excluded.image_url
         where (players.real_name, players.country, players.role, players.image_url)
           is distinct from
           (excluded.real_name, excluded.country, excluded.role, excluded.image_url)",
    )
    .bind(players.iter().map(|p| p.name.clone()).collect::<Vec<_>>())
    .bind(
        players
            .iter()
            .map(|p| p.real_name.clone())
            .collect::<Vec<_>>(),
    )
    .bind(
        players
            .iter()
            .map(|p| p.country.clone())
            .collect::<Vec<_>>(),
    )
    .bind(players.iter().map(|p| p.role.clone()).collect::<Vec<_>>())
    .bind(
        players
            .iter()
            .map(|p| p.image_url.clone())
            .collect::<Vec<_>>(),
    )
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

async fn upsert_raw_events(tx: &mut Transaction<'_, Postgres>, data: &Normalized) -> Result<u64> {
    let events = &data.raw_events;
    let result = sqlx::query(
        "insert into raw_events (id, date, player, team, team_raw, direction, role, preload,
           status, is_sub, is_loan, is_retired, news_id, line_in_news, source_url, player_raw)
         select * from unnest($1::text[], $2::date[], $3::text[], $4::text[], $5::text[],
           $6::text[], $7::text[], $8::text[], $9::text[], $10::bool[], $11::bool[], $12::bool[],
           $13::text[], $14::int[], $15::text[], $16::text[])
         on conflict (id) do update set
           date = excluded.date, player = excluded.player, player_raw = excluded.player_raw,
           team = excluded.team,
           team_raw = excluded.team_raw, direction = excluded.direction, role = excluded.role,
           preload = excluded.preload, status = excluded.status, is_sub = excluded.is_sub,
           is_loan = excluded.is_loan, is_retired = excluded.is_retired,
           news_id = excluded.news_id, line_in_news = excluded.line_in_news,
           source_url = excluded.source_url, fetched_at = now(), deleted_at = null
         where (raw_events.date, raw_events.player, raw_events.team, raw_events.team_raw,
             raw_events.direction, raw_events.role, raw_events.preload, raw_events.status,
             raw_events.is_sub, raw_events.is_loan, raw_events.is_retired, raw_events.news_id,
             raw_events.line_in_news, raw_events.source_url, raw_events.player_raw,
             raw_events.deleted_at)
           is distinct from
           (excluded.date, excluded.player, excluded.team, excluded.team_raw,
             excluded.direction, excluded.role, excluded.preload, excluded.status,
             excluded.is_sub, excluded.is_loan, excluded.is_retired, excluded.news_id,
             excluded.line_in_news, excluded.source_url, excluded.player_raw, null)",
    )
    .bind(events.iter().map(|e| e.id.clone()).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.date).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.player.clone()).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.team.clone()).collect::<Vec<_>>())
    .bind(
        events
            .iter()
            .map(|e| e.team_raw.clone())
            .collect::<Vec<_>>(),
    )
    .bind(
        events
            .iter()
            .map(|e| e.direction.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(events.iter().map(|e| e.role.clone()).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.preload.clone()).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.status.clone()).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.is_sub).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.is_loan).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.is_retired).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.news_id.clone()).collect::<Vec<_>>())
    .bind(events.iter().map(|e| e.line_in_news).collect::<Vec<_>>())
    .bind(
        events
            .iter()
            .map(|e| e.source_url.clone())
            .collect::<Vec<_>>(),
    )
    .bind(
        events
            .iter()
            .map(|e| e.player_raw.clone())
            .collect::<Vec<_>>(),
    )
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

async fn mark_deleted(
    tx: &mut Transaction<'_, Postgres>,
    data: &Normalized,
    window: &Window<'_>,
) -> Result<Vec<String>> {
    let players = sqlx::query_scalar(
        "update raw_events set deleted_at = now()
         where deleted_at is null
           and date >= $1
           and (player_raw = any($2) or team_raw = any($3))
           and not (id = any($4))
         returning player",
    )
    .bind(window.since)
    .bind(window.players)
    .bind(window.teams)
    .bind(
        data.raw_events
            .iter()
            .map(|e| e.id.clone())
            .collect::<Vec<_>>(),
    )
    .fetch_all(&mut **tx)
    .await?;
    Ok(players)
}

/// Re-pairs every live event of `players` and syncs their transfers, keyed by source events.
async fn rebuild_transfers(tx: &mut Transaction<'_, Postgres>, players: &[String]) -> Result<u64> {
    let events = load_events(tx, players).await?;
    let renames = load_renames(tx).await?;
    let transfers = pairing::pair(&events, &renames);
    let column = |f: fn(&Transfer) -> Option<String>| transfers.iter().map(f).collect::<Vec<_>>();

    let removed = sqlx::query(
        "delete from transfers t
         where t.player = any($1)
           and not exists (
             select 1 from unnest($2::text[], $3::text[]) as k(leave_event, join_event)
             where k.leave_event is not distinct from t.leave_event
               and k.join_event is not distinct from t.join_event)",
    )
    .bind(players)
    .bind(column(|t| t.leave_event.clone()))
    .bind(column(|t| t.join_event.clone()))
    .execute(&mut **tx)
    .await?;
    let upserted = sqlx::query(
        "insert into transfers (player, from_team, to_team, role, kind, date, leave_event, join_event)
         select * from unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
           $6::date[], $7::text[], $8::text[])
         on conflict (leave_event, join_event) do update set
           player = excluded.player, from_team = excluded.from_team, to_team = excluded.to_team,
           role = excluded.role, kind = excluded.kind, date = excluded.date, updated_at = now()
         where (transfers.player, transfers.from_team, transfers.to_team, transfers.role,
             transfers.kind, transfers.date)
           is distinct from
           (excluded.player, excluded.from_team, excluded.to_team, excluded.role,
             excluded.kind, excluded.date)",
    )
    .bind(column(|t| Some(t.player.clone())))
    .bind(column(|t| t.from_team.clone()))
    .bind(column(|t| t.to_team.clone()))
    .bind(column(|t| t.role.clone()))
    .bind(column(|t| Some(t.kind.as_str().to_owned())))
    .bind(transfers.iter().map(|t| t.date).collect::<Vec<_>>())
    .bind(column(|t| t.leave_event.clone()))
    .bind(column(|t| t.join_event.clone()))
    .execute(&mut **tx)
    .await?;
    Ok(removed.rows_affected() + upserted.rows_affected())
}

async fn load_events(
    tx: &mut Transaction<'_, Postgres>,
    players: &[String],
) -> Result<Vec<RawEvent>> {
    let rows = sqlx::query(
        "select id, date, player, player_raw, team, team_raw, direction, role, preload, status,
           is_sub, is_loan, is_retired, news_id, line_in_news, source_url
         from raw_events
         where player = any($1) and deleted_at is null",
    )
    .bind(players)
    .fetch_all(&mut **tx)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(RawEvent {
                id: row.try_get("id")?,
                date: row.try_get("date")?,
                player: row.try_get("player")?,
                player_raw: row.try_get("player_raw")?,
                team: row.try_get("team")?,
                team_raw: row.try_get("team_raw")?,
                direction: match row.try_get::<&str, _>("direction")? {
                    "join" => Direction::Join,
                    _ => Direction::Leave,
                },
                role: row.try_get("role")?,
                preload: row.try_get("preload")?,
                status: row.try_get("status")?,
                is_sub: row.try_get("is_sub")?,
                is_loan: row.try_get("is_loan")?,
                is_retired: row.try_get("is_retired")?,
                news_id: row.try_get("news_id")?,
                line_in_news: row.try_get("line_in_news")?,
                source_url: row.try_get("source_url")?,
            })
        })
        .collect()
}
