mod common;

use chrono::{Days, NaiveDate, Utc};
use ingestor::db::{self, Window};
use ingestor::normalize::{Normalized, normalize};
use ingestor::rumors::{self, Rumor, Source, Status};
use sqlx::PgPool;

#[test]
fn the_committed_rumors_file_is_valid() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/rumors.yaml");
    let yaml = std::fs::read_to_string(path).unwrap();
    if let Err(error) = rumors::parse(&yaml, rumors::latest_report_date(Utc::now())) {
        panic!("{error:#}");
    }
}

async fn write(pool: &PgPool, data: &Normalized) {
    let players: Vec<String> = data
        .raw_events
        .iter()
        .map(|e| e.player_raw.clone())
        .collect();
    let teams: Vec<String> = data.raw_events.iter().map(|e| e.team_raw.clone()).collect();
    let window = Window {
        since: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        players: &players,
        teams: &teams,
        allow_mass_delete: false,
    };
    db::write(pool, data, &window).await.unwrap();
}

fn rumor(id: &str, player: &str, to_team: &str, reported_at: NaiveDate, status: Status) -> Rumor {
    Rumor {
        id: id.to_owned(),
        player: player.to_owned(),
        from_team: None,
        to_team: Some(to_team.to_owned()),
        role: None,
        kind: None,
        reported_at,
        sources: vec![Source {
            url: "https://example.com".to_owned(),
            outlet: None,
        }],
        reliability: 3,
        status,
        notes: None,
    }
}

async fn statuses(pool: &PgPool) -> Vec<(String, String, Option<String>)> {
    sqlx::query_as("select id, status, transfer_id::text from rumors order by id")
        .fetch_all(pool)
        .await
        .unwrap()
}

/// Needs a disposable database: the test truncates every ingestor table.
#[tokio::test]
async fn rumors_sync_idempotently_and_survive_transfer_rebuilds() {
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
    write(&pool, &data).await;

    let (transfer_id, player, to_team, date, join_event): (
        String,
        String,
        String,
        NaiveDate,
        String,
    ) = sqlx::query_as(
        "select id::text, player, to_team, date, join_event from transfers
             where kind = 'transfer' order by date, id limit 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let today = date + Days::new(100);
    let mut list = vec![
        rumor(
            "a-confirmed",
            &player,
            &to_team,
            date - Days::new(1),
            Status::Rumor,
        ),
        rumor("b-denied", &player, &to_team, date, Status::Denied),
        rumor(
            "c-contradicted",
            &player,
            "Nowhere",
            date - Days::new(1),
            Status::Rumor,
        ),
        rumor(
            "d-stale",
            "Nobody",
            "Nowhere",
            date - Days::new(1),
            Status::Rumor,
        ),
        rumor("e-open", &player, "Nowhere", today, Status::Rumor),
    ];
    assert_eq!(db::sync_rumors(&pool, &list, today).await.unwrap(), 5);
    let expected = |s: &str, t: Option<&str>| (s.to_owned(), t.map(str::to_owned));
    let rows = statuses(&pool).await;
    let got: Vec<_> = rows
        .iter()
        .map(|(_, s, t)| (s.clone(), t.clone()))
        .collect();
    assert_eq!(
        got,
        [
            expected("confirmed", Some(&transfer_id)),
            expected("denied", None),
            expected("contradicted", None),
            expected("stale", None),
            expected("rumor", None),
        ]
    );
    assert_eq!(
        db::sync_rumors(&pool, &list, today).await.unwrap(),
        0,
        "second sync must change nothing"
    );

    list.pop();
    assert_eq!(db::sync_rumors(&pool, &list, today).await.unwrap(), 1);
    assert_eq!(
        statuses(&pool).await.len(),
        4,
        "rumors removed from YAML are deleted"
    );

    data.raw_events.retain(|e| e.id != join_event);
    write(&pool, &data).await;
    let link: Option<String> =
        sqlx::query_scalar("select transfer_id::text from rumors where id = 'a-confirmed'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        link, None,
        "rebuilding transfers unlinks instead of failing"
    );
    db::sync_rumors(&pool, &list, today).await.unwrap();
    assert_eq!(statuses(&pool).await[0].1, "stale");
}
