use std::collections::{HashMap, HashSet};
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
const MAX_TITLES: usize = 50;

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
    pub join_on: String,
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
        for (key, value) in [
            ("join_on", &self.join_on),
            ("where", &self.where_clause),
            ("order_by", &self.order_by),
        ] {
            if !value.is_empty() {
                params.push((key, value.clone()));
            }
        }
        params
    }
}

/// `Ok(true)` when Leaguepedia rate-limited the request; other errors and warnings fail.
fn rate_limited(body: &Value) -> Result<bool> {
    if let Some(error) = body.get("error") {
        if error.get("code").and_then(Value::as_str) == Some("ratelimited") {
            return Ok(true);
        }
        bail!("Leaguepedia error: {error}");
    }
    if let Some(warnings) = body.get("warnings") {
        bail!("Leaguepedia warning: {warnings}");
    }
    Ok(false)
}

/// `None` when Leaguepedia rate-limited the request.
pub fn parse_page(body: &Value) -> Result<Option<Vec<Row>>> {
    if rate_limited(body)? {
        return Ok(None);
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
        .map(Some)
}

pub fn files_params(titles: &[String]) -> Vec<(&'static str, String)> {
    let titles = titles.iter().map(|title| format!("File:{title}"));
    vec![
        ("action", "query".to_owned()),
        ("format", "json".to_owned()),
        ("formatversion", "2".to_owned()),
        ("redirects", "1".to_owned()),
        ("prop", "imageinfo".to_owned()),
        ("titles", titles.collect::<Vec<_>>().join("|")),
    ]
}

/// One `File` → `Resolved` row per requested file that exists, following MediaWiki's title
/// normalisation and file redirects; missing files get no row.
pub fn parse_files(requested: &[String], body: &Value) -> Result<Option<Vec<Row>>> {
    if rate_limited(body)? {
        return Ok(None);
    }
    let query = body
        .get("query")
        .context("response has no `query` object")?;
    let mapping = |key: &str| -> HashMap<&str, &str> {
        query
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|entry| Some((entry.get("from")?.as_str()?, entry.get("to")?.as_str()?)))
            .collect()
    };
    let (normalized, redirects) = (mapping("normalized"), mapping("redirects"));
    let existing: HashSet<&str> = query
        .get("pages")
        .and_then(Value::as_array)
        .context("response has no `pages` array")?
        .iter()
        .filter(|page| {
            page.get("imagerepository")
                .and_then(Value::as_str)
                .is_some_and(|repository| !repository.is_empty())
        })
        .filter_map(|page| page.get("title")?.as_str())
        .collect();
    let rows = requested
        .iter()
        .filter_map(|file| {
            let title = format!("File:{file}");
            let title = normalized.get(title.as_str()).copied().unwrap_or(&title);
            let title = redirects.get(title).copied().unwrap_or(title);
            let resolved = existing.get(title)?.strip_prefix("File:")?;
            let mut row = Row::new();
            row.insert("File".into(), file.as_str().into());
            row.insert("Resolved".into(), resolved.into());
            Some(row)
        })
        .collect();
    Ok(Some(rows))
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

    /// Resolves file names in batches of the anonymous `titles` limit.
    pub async fn resolve_files(&mut self, files: &[String]) -> Result<Vec<Row>> {
        let mut rows = Vec::new();
        for chunk in files.chunks(MAX_TITLES) {
            let params = files_params(chunk);
            rows.extend(self.fetch(&params, |body| parse_files(chunk, body)).await?);
        }
        Ok(rows)
    }

    async fn fetch_page(&mut self, query: &Query, offset: usize) -> Result<Vec<Row>> {
        let rows = self.fetch(&query.params(offset), parse_page).await?;
        tracing::debug!(table = %query.tables, offset, rows = rows.len(), "fetched page");
        Ok(rows)
    }

    /// `parse` returns `None` when Leaguepedia rate-limits the request.
    async fn fetch<T>(
        &mut self,
        params: &[(&'static str, String)],
        parse: impl Fn(&Value) -> Result<Option<T>>,
    ) -> Result<T> {
        for attempt in 0..=MAX_RETRIES {
            self.throttle().await;
            let reason = match self.request(params).await.and_then(|body| parse(&body)) {
                Ok(Some(parsed)) => return Ok(parsed),
                Ok(None) => "rate limited".to_owned(),
                Err(error) if is_transient(&error) => format!("{error:#}"),
                Err(error) => return Err(error),
            };
            let wait = BACKOFF_BASE * 2u32.pow(attempt);
            tracing::warn!(attempt, wait_secs = wait.as_secs(), %reason, "backing off");
            sleep(wait).await;
        }
        bail!("Leaguepedia still failing after {MAX_RETRIES} retries")
    }

    async fn request(&self, params: &[(&'static str, String)]) -> Result<Value> {
        Ok(self
            .http
            .post(API_URL)
            .form(params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
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
        let rows = parse_page(&body).unwrap().expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["Player"], "Caps");
    }

    #[test]
    fn detects_rate_limit() {
        let body = json!({"error": {"code": "ratelimited", "info": "rate limit exceeded"}});
        assert_eq!(parse_page(&body).unwrap(), None);
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

    #[test]
    fn resolves_file_redirects_and_skips_missing_files() {
        let body = json!({"batchcomplete": true, "query": {
            "redirects": [{"from": "File:HANJIN BRIONlogo square.png", "to": "File:BRIONlogo square.png"}],
            "pages": [
                {"ns": 6, "title": "File:Nonexistent Teamlogo square.png", "missing": true, "imagerepository": ""},
                {"pageid": 293550, "ns": 6, "title": "File:T1logo square.png", "imagerepository": "local"},
                {"pageid": 438424, "ns": 6, "title": "File:BRIONlogo square.png", "imagerepository": "local"}
            ]
        }});
        let requested = [
            "T1logo square.png".to_owned(),
            "HANJIN BRIONlogo square.png".to_owned(),
            "Nonexistent Teamlogo square.png".to_owned(),
        ];
        let rows = parse_files(&requested, &body).unwrap().unwrap();
        let resolved: Vec<_> = rows
            .iter()
            .map(|row| {
                (
                    row["File"].as_str().unwrap(),
                    row["Resolved"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            resolved,
            [
                ("T1logo square.png", "T1logo square.png"),
                ("HANJIN BRIONlogo square.png", "BRIONlogo square.png"),
            ]
        );
    }

    #[test]
    fn file_queries_follow_normalised_titles_and_detect_rate_limits() {
        let body = json!({"query": {
            "normalized": [{"from": "File:G2_Esportslogo_square.png", "to": "File:G2 Esportslogo square.png"}],
            "pages": [{"title": "File:G2 Esportslogo square.png", "imagerepository": "local"}]
        }});
        let rows = parse_files(&["G2_Esportslogo_square.png".to_owned()], &body)
            .unwrap()
            .unwrap();
        assert_eq!(rows[0]["Resolved"], "G2 Esportslogo square.png");
        let limited = json!({"error": {"code": "ratelimited"}});
        assert_eq!(parse_files(&[], &limited).unwrap(), None);
    }
}
