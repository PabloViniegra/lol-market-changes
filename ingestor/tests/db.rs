mod common;

use std::collections::HashMap;

use chrono::NaiveDate;
use ingestor::db::{self, Window, WriteStats};
use ingestor::fetch::ScopeRows;
use ingestor::normalize::{Normalized, normalize};
use ingestor::pairing;

fn raw_names(data: &Normalized) -> (Vec<String>, Vec<String>) {
    let players = data
        .raw_events
        .iter()
        .map(|e| e.player_raw.clone())
        .collect();
    let teams = data.raw_events.iter().map(|e| e.team_raw.clone()).collect();
    (players, teams)
}

async fn write(pool: &sqlx::PgPool, data: &Normalized) -> WriteStats {
    let (players, teams) = raw_names(data);
    let window = Window {
        since: early(),
        players: &players,
        teams: &teams,
        allow_mass_delete: false,
    };
    db::write(pool, data, &window).await.unwrap()
}

/// Needs a disposable database: the test truncates every ingestor table.
#[tokio::test]
async fn writes_are_idempotent_and_track_wiki_deletions() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL not set, skipping database test");
        return;
    };
    let pool = db::connect(&url).await.unwrap();
    sqlx::query("truncate rumors, transfers, raw_events, team_seasons, players, teams")
        .execute(&pool)
        .await
        .unwrap();

    let mut data = normalize(&common::probe_snapshot()).unwrap();
    let (players, teams) = raw_names(&data);
    let first = write(&pool, &data).await;
    assert_eq!(first.raw_events, data.raw_events.len() as u64);
    assert_eq!(first.players, data.players.len() as u64);
    assert_eq!(first.teams, data.teams.len() as u64);
    let transfers = pairing::pair(&data.raw_events, &HashMap::new());
    assert!(!transfers.is_empty());
    assert_eq!(first.transfers, transfers.len() as u64);

    assert_eq!(
        write(&pool, &data).await,
        WriteStats::default(),
        "second run must change nothing"
    );

    let removed = data.raw_events.pop().unwrap();
    let after_removal = write(&pool, &data).await;
    assert_eq!(after_removal.deleted_events, 1);
    assert!(
        after_removal.transfers > 0,
        "transfers of a deleted event are rebuilt"
    );
    let stale: i64 =
        sqlx::query_scalar("select count(*) from transfers where $1 in (leave_event, join_event)")
            .bind(&removed.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stale, 0);
    assert_eq!(write(&pool, &data).await, WriteStats::default());

    data.raw_events.push(removed.clone());
    let restored = write(&pool, &data).await;
    assert_eq!(restored.raw_events, 1);
    let deleted_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("select deleted_at from raw_events where id = $1")
            .bind(&removed.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(deleted_at, None);

    data.raw_events.pop();
    let before_window = Window {
        since: removed.date.succ_opt().unwrap(),
        players: &players,
        teams: &teams,
        allow_mass_delete: false,
    };
    let stats = db::write(&pool, &data, &before_window).await.unwrap();
    assert_eq!(
        stats.deleted_events, 0,
        "events before `since` are not re-fetched"
    );
    let other_names = Window {
        since: early(),
        players: &[],
        teams: &[],
        allow_mass_delete: false,
    };
    let stats = db::write(&pool, &data, &other_names).await.unwrap();
    assert_eq!(
        stats.deleted_events, 0,
        "events of names not queried are not re-fetched"
    );

    let deleted_count = "select count(*) from raw_events where deleted_at is not null";
    let deleted_before: i64 = sqlx::query_scalar(deleted_count)
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut empty = normalize(&common::probe_snapshot()).unwrap();
    empty.raw_events.clear();
    let full_window = Window {
        since: early(),
        players: &players,
        teams: &teams,
        allow_mass_delete: false,
    };
    let error = db::write(&pool, &empty, &full_window).await.unwrap_err();
    assert!(format!("{error:#}").contains("refusing"), "{error:#}");
    let deleted_after: i64 = sqlx::query_scalar(deleted_count)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        deleted_after, deleted_before,
        "an aborted run deletes nothing"
    );

    let forced = Window {
        allow_mass_delete: true,
        ..full_window
    };
    let stats = db::write(&pool, &empty, &forced).await.unwrap();
    assert!(
        stats.deleted_events >= 5,
        "the override applies the deletions"
    );
}

fn early() -> NaiveDate {
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
}

#[tokio::test]
async fn a_failed_run_preserves_the_last_successful_refresh() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL not set, skipping database test");
        return;
    };
    let pool = db::connect(&url).await.unwrap();
    sqlx::query("truncate sync_state")
        .execute(&pool)
        .await
        .unwrap();
    let stats = serde_json::json!({});
    let freshness = "select last_run_status, last_run_at,
        (to_jsonb(sync_state)->>'last_successful_run_at')::timestamptz from sync_state";

    db::record_run(&pool, None, "error", &stats).await.unwrap();
    let (_, _, success): (
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(freshness).fetch_one(&pool).await.unwrap();
    assert_eq!(success, None);

    db::record_run(&pool, Some(early()), "ok", &stats)
        .await
        .unwrap();
    let (status, attempt, success): (
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(freshness).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "ok");
    assert_eq!(success, Some(attempt));

    db::record_run(&pool, None, "error", &stats).await.unwrap();
    let (status, _, after_failure): (
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(freshness).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "error");
    assert_eq!(after_failure, success);
}

#[tokio::test]
async fn scope_rows_are_reused_only_for_later_windows() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL not set, skipping database test");
        return;
    };
    let pool = db::connect(&url).await.unwrap();
    sqlx::query("truncate leaguepedia_cache")
        .execute(&pool)
        .await
        .unwrap();
    let snapshot = common::probe_snapshot();
    let rows = ScopeRows {
        tournaments: snapshot.tournaments,
        tournament_rosters: snapshot.tournament_rosters,
        team_redirects: snapshot.team_redirects,
    };
    let since = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap();
    assert_eq!(db::load_scope_rows(&pool, since).await.unwrap(), None);

    db::save_scope_rows(&pool, since, &rows).await.unwrap();
    assert_eq!(
        db::load_scope_rows(&pool, since).await.unwrap(),
        Some(rows.clone())
    );
    let later = since.succ_opt().unwrap();
    assert!(db::load_scope_rows(&pool, later).await.unwrap().is_some());
    let earlier = since.pred_opt().unwrap();
    assert_eq!(db::load_scope_rows(&pool, earlier).await.unwrap(), None);

    sqlx::query("update leaguepedia_cache set fetched_at = now() - interval '25 hours'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(db::load_scope_rows(&pool, since).await.unwrap(), None);
}
