use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};

use crate::normalize::Normalized;

#[derive(Debug, Default, PartialEq)]
pub struct WriteStats {
    pub teams: u64,
    pub team_seasons: u64,
    pub players: u64,
    pub raw_events: u64,
    pub deleted_events: u64,
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
        raw_events: upsert_raw_events(&mut tx, data).await?,
        deleted_events: mark_deleted(&mut tx, data, window).await?,
    };
    tx.commit().await?;
    Ok(stats)
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
        "insert into teams (name, short, league, region, org, logo_url)
         select * from unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[])
         on conflict (name) do update set
           short = excluded.short, league = excluded.league, region = excluded.region,
           org = excluded.org, logo_url = excluded.logo_url
         where (teams.short, teams.league, teams.region, teams.org, teams.logo_url)
           is distinct from
           (excluded.short, excluded.league, excluded.region, excluded.org, excluded.logo_url)",
    )
    .bind(teams.iter().map(|t| t.name.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.short.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.league.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.region.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.org.clone()).collect::<Vec<_>>())
    .bind(teams.iter().map(|t| t.logo_url.clone()).collect::<Vec<_>>())
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

async fn upsert_team_seasons(tx: &mut Transaction<'_, Postgres>, data: &Normalized) -> Result<u64> {
    let seasons = &data.team_seasons;
    let result = sqlx::query(
        "insert into team_seasons (team, league, tournament, date_start, date_end)
         select * from unnest($1::text[], $2::text[], $3::text[], $4::date[], $5::date[])
         on conflict (team, tournament) do update set
           league = excluded.league, date_start = excluded.date_start, date_end = excluded.date_end
         where (team_seasons.league, team_seasons.date_start, team_seasons.date_end)
           is distinct from (excluded.league, excluded.date_start, excluded.date_end)",
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
) -> Result<u64> {
    let result = sqlx::query(
        "update raw_events set deleted_at = now()
         where deleted_at is null
           and date >= $1
           and (player_raw = any($2) or team_raw = any($3))
           and not (id = any($4))",
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
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}
