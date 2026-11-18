mod common;

use chrono::NaiveDate;
use ingestor::db::{self, Window, WriteStats};
use ingestor::normalize::{Normalized, normalize};

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
    sqlx::query("truncate raw_events, team_seasons, players, teams, sync_state")
        .execute(&pool)
        .await
        .unwrap();

    let mut data = normalize(&common::probe_snapshot()).unwrap();
    let (players, teams) = raw_names(&data);
    let first = write(&pool, &data).await;
    assert_eq!(first.raw_events, data.raw_events.len() as u64);
    assert_eq!(first.players, data.players.len() as u64);
    assert_eq!(first.teams, data.teams.len() as u64);

    assert_eq!(
        write(&pool, &data).await,
        WriteStats::default(),
        "second run must change nothing"
    );

    let removed = data.raw_events.pop().unwrap();
    let after_removal = write(&pool, &data).await;
    assert_eq!(after_removal.deleted_events, 1);
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
    };
    let stats = db::write(&pool, &data, &other_names).await.unwrap();
    assert_eq!(
        stats.deleted_events, 0,
        "events of names not queried are not re-fetched"
    );
}

fn early() -> NaiveDate {
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
}
