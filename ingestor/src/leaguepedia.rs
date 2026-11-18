use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};
use tokio::time::{Instant, sleep, sleep_until};

pub const API_URL: &str = "https://lol.fandom.com/api.php";
pub const USER_AGENT: &str = concat!(
    "lol-market-changes-ingestor/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/PabloViniegra/lol-market-changes)"
);
pub const MAX_LIMIT: usize = 500;

// Anonymous requests hit `ratelimited` after ~5 calls at 1.2 s or 2 s spacing (M0 probe),
// which suggests a budget of about 10 requests per minute.
const MIN_INTERVAL: Duration = Duration::from_secs(6);
const BACKOFF_BASE: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 4;

pub type Row = Map<String, Value>;

#[derive(Debug, Clone, Default)]
pub struct Query {
    pub tables: String,
    pub fields: String,
    pub where_clause: String,
    pub order_by: String,
}

impl Query {
    pub fn params(&self, offset: usize) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("action", "cargoquery".to_owned()),
            ("format", "json".to_owned()),
            ("tables", self.tables.clone()),
            ("fields", self.fields.clone()),
            ("limit", MAX_LIMIT.to_string()),
            ("offset", offset.to_string()),
        ];
        for (key, value) in [("where", &self.where_clause), ("order_by", &self.order_by)] {
            if !value.is_empty() {
                params.push((key, value.clone()));
            }
        }
        params
    }
}

#[derive(Debug, PartialEq)]
pub enum Page {
    Rows(Vec<Row>),
    RateLimited,
}

pub fn parse_page(body: &Value) -> Result<Page> {
    if let Some(error) = body.get("error") {
        if error.get("code").and_then(Value::as_str) == Some("ratelimited") {
            return Ok(Page::RateLimited);
        }
        bail!("Leaguepedia error: {error}");
    }
    if let Some(warnings) = body.get("warnings") {
        bail!("Leaguepedia warning: {warnings}");
    }
    body.get("cargoquery")
        .and_then(Value::as_array)
        .context("response has no `cargoquery` array")?
        .iter()
        .map(|row| {
            row.get("title")
                .and_then(Value::as_object)
                .cloned()
                .context("row has no `title` object")
        })
        .collect::<Result<_>>()
        .map(Page::Rows)
}

pub fn in_list<S: AsRef<str>>(field: &str, values: &[S]) -> Result<String> {
    let quoted = values
        .iter()
        .map(|value| {
            let value = value.as_ref();
            if value.contains('"') {
                bail!("cannot quote value containing a double quote: {value}");
            }
            Ok(format!("\"{value}\""))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(format!("{field} IN ({})", quoted.join(",")))
}

/// Network failures, timeouts, 429 and 5xx; Cargo query errors and 4xx are not retried.
fn is_transient(error: &anyhow::Error) -> bool {
    error.downcast_ref::<reqwest::Error>().is_some_and(|e| {
        e.is_timeout()
            || e.is_connect()
            || e.is_body()
            || e.status().is_some_and(|status| {
                status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS
            })
    })
}

pub struct Client {
    http: reqwest::Client,
    last_request: Option<Instant>,
}

impl Client {
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            last_request: None,
        })
    }

    pub async fn query_all(&mut self, query: &Query) -> Result<Vec<Row>> {
        let mut rows = Vec::new();
        loop {
            let page = self.fetch_page(query, rows.len()).await?;
            let done = page.len() < MAX_LIMIT;
            rows.extend(page);
            if done {
                return Ok(rows);
            }
        }
    }

    async fn fetch_page(&mut self, query: &Query, offset: usize) -> Result<Vec<Row>> {
        for attempt in 0..=MAX_RETRIES {
            self.throttle().await;
            let reason = match self.request(query, offset).await {
                Ok(Page::Rows(rows)) => {
                    tracing::debug!(table = %query.tables, offset, rows = rows.len(), "fetched page");
                    return Ok(rows);
                }
                Ok(Page::RateLimited) => "rate limited".to_owned(),
                Err(error) if is_transient(&error) => format!("{error:#}"),
                Err(error) => return Err(error),
            };
            let wait = BACKOFF_BASE * 2u32.pow(attempt);
            tracing::warn!(attempt, wait_secs = wait.as_secs(), %reason, "backing off");
            sleep(wait).await;
        }
        bail!("Leaguepedia still failing after {MAX_RETRIES} retries")
    }

    async fn request(&self, query: &Query, offset: usize) -> Result<Page> {
        let body: Value = self
            .http
            .post(API_URL)
            .form(&query.params(offset))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        parse_page(&body)
    }

    async fn throttle(&mut self) {
        if let Some(last) = self.last_request {
            sleep_until(last + MIN_INTERVAL).await;
        }
        self.last_request = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_rows_from_title_objects() {
        let body = json!({"cargoquery": [{"title": {"Player": "Caps", "Direction": "Join"}}]});
        let Page::Rows(rows) = parse_page(&body).unwrap() else {
            panic!("expected rows");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["Player"], "Caps");
    }

    #[test]
    fn detects_rate_limit() {
        let body = json!({"error": {"code": "ratelimited", "info": "rate limit exceeded"}});
        assert_eq!(parse_page(&body).unwrap(), Page::RateLimited);
    }

    #[test]
    fn other_errors_and_warnings_fail() {
        let error = json!({"error": {"code": "internal_api_error_MWException"}});
        assert!(parse_page(&error).is_err());
        let warning = json!({"warnings": {"cargoquery": {"*": "limit must be between 1 and 500"}}, "cargoquery": []});
        assert!(parse_page(&warning).is_err());
    }

    #[tokio::test]
    async fn connection_failures_are_transient_but_query_errors_are_not() {
        let refused = reqwest::get("http://127.0.0.1:1").await.unwrap_err();
        assert!(is_transient(&refused.into()));
        let query_error = parse_page(&json!({"error": {"code": "MWException"}})).unwrap_err();
        assert!(!is_transient(&query_error));
    }

    #[test]
    fn builds_in_list_and_rejects_double_quotes() {
        assert_eq!(
            in_list("Team", &["G2 Esports", "T1"]).unwrap(),
            r#"Team IN ("G2 Esports","T1")"#
        );
        assert!(in_list("Team", &[r#"Bad "Name""#]).is_err());
    }

    #[test]
    fn params_skip_empty_clauses() {
        let query = Query {
            tables: "RosterChanges".into(),
            fields: "Player".into(),
            ..Default::default()
        };
        let keys: Vec<_> = query.params(500).into_iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            ["action", "format", "tables", "fields", "limit", "offset"]
        );
    }
}
