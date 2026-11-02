//! Stateful Graph boundary, reachable only on loopback. All credentials/content are synthetic.
// Shared by core and HTTP test binaries, which inspect different subsets of the recorded calls.
#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
use axum::{
    Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
pub const TOKEN: &str = "fake-page-token-zebrafish";
pub const SCOPES: &[&str] = &[
    "pages_show_list",
    "pages_read_engagement",
    "pages_manage_posts",
    "instagram_basic",
    "instagram_content_publish",
];
#[derive(Clone, Default)]
pub struct Config {
    pub wrong_state: bool,
    pub deny: bool,
    pub missing_permission: bool,
    pub granted_permissions: Option<Vec<String>>,
    pub wrong_page: bool,
    pub second_page: bool,
    pub second_instagram: bool,
    pub no_tasks: bool,
    pub no_instagram: bool,
    pub container_status: Option<String>,
    pub ready_after: usize,
    pub fault_path: Option<String>,
    pub fault_status: u16,
    pub fault_code: u16,
    pub fault_after: usize,
    pub empty_body: bool,
    pub malformed: bool,
    pub hang: bool,
    pub bad_permalink: bool,
    pub photo_only: bool,
}
#[derive(Clone, Debug)]
pub struct Call {
    pub path: String,
    pub method: String,
    pub headers: HeaderMap,
    pub query: HashMap<String, String>,
    pub body: Value,
}
struct Inner {
    config: Mutex<Config>,
    calls: Mutex<Vec<Call>>,
    containers: Mutex<Vec<String>>,
}
pub struct FakeMeta {
    base: String,
    inner: Arc<Inner>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for FakeMeta {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl FakeMeta {
    pub fn base(&self) -> &str {
        &self.base
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn configure(&self, c: Config) {
        *self.inner.config.lock().unwrap() = c;
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn calls(&self, path: &str) -> Vec<Call> {
        self.inner
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.path == path)
            .cloned()
            .collect()
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn all(&self) -> Vec<Call> {
        self.inner.calls.lock().unwrap().clone()
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn start(config: Config) -> FakeMeta {
    let inner = Arc::new(Inner {
        config: Mutex::new(config),
        calls: Mutex::new(vec![]),
        containers: Mutex::new(vec![]),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v26.0/dialog/oauth", get(authorize))
        .route("/v26.0/{*path}", get(graph_get).post(graph_post))
        .with_state(inner.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    FakeMeta { base, inner, task }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn authorize(
    State(i): State<Arc<Inner>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    assert_eq!(q["config_id"], "config-1");
    assert_eq!(q["response_type"], "code");
    assert!(!q.contains_key("scope"));
    let c = i.config.lock().unwrap().clone();
    let mut url = reqwest::Url::parse(&q["redirect_uri"]).unwrap();
    url.query_pairs_mut()
        .append_pair("state", if c.wrong_state { "wrong" } else { &q["state"] });
    url.query_pairs_mut().append_pair(
        if c.deny { "error" } else { "code" },
        if c.deny { "access_denied" } else { "fake-code" },
    );
    Redirect::temporary(url.as_str()).into_response()
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn graph_get(
    State(i): State<Arc<Inner>>,
    Path(path): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    h: HeaderMap,
) -> Response {
    record(&i, &path, "GET", h.clone(), q.clone(), Value::Null);
    if let Some(response) = fault(&i, &path).await {
        return response;
    }
    let c = i.config.lock().unwrap().clone();
    let answer = match path.as_str() {
        "oauth/access_token" => {
            assert_eq!(q["client_secret"], "fake-app-secret");
            let long = q
                .get("grant_type")
                .is_some_and(|s| s == "fb_exchange_token");
            if long {
                assert_eq!(q["fb_exchange_token"], "fake-user-short");
            } else {
                assert_eq!(q["code"], "fake-code");
                assert!(q["redirect_uri"].starts_with("http://127.0.0.1:"));
            }
            json!({"access_token":if long {"fake-user-long"} else {"fake-user-short"},"expires_in":5184000})
        }
        "me/permissions" => {
            let permissions = c
                .granted_permissions
                .clone()
                .unwrap_or_else(|| SCOPES.iter().map(|scope| (*scope).to_owned()).collect());
            json!({"data":permissions.iter().enumerate().map(|(n,p)|json!({"permission":p,"status":if c.missing_permission && n==0 {"declined"}else{"granted"}})).collect::<Vec<_>>()})
        }
        "me/accounts" => {
            assert_eq!(h["authorization"], "Bearer fake-user-long");
            json!({"data":[{"id":if c.wrong_page {"999"}else if c.second_page {"111"}else{"101"},"access_token":TOKEN,"tasks":if c.no_tasks {vec!["ANALYZE"]} else {vec!["CREATE_CONTENT","MANAGE","MODERATE"]},"instagram_business_account":if c.no_instagram {Value::Null} else {json!({"id":if c.second_instagram {"212"}else{"202"}})}}]})
        }
        "303" => {
            json!({"permalink":if c.bad_permalink {"https://evil.invalid/p/303/"}else{"https://www.instagram.com/p/fake303/"}})
        }
        _ => {
            assert!(i.containers.lock().unwrap().iter().any(|id| id == &path));
            let checks = i
                .calls
                .lock()
                .unwrap()
                .iter()
                .filter(|call| call.path == path)
                .count();
            let status = if checks <= c.ready_after {
                "IN_PROGRESS"
            } else {
                c.container_status.as_deref().unwrap_or("FINISHED")
            };
            json!({"status_code":status})
        }
    };
    axum::Json(answer).into_response()
}
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn graph_post(
    State(i): State<Arc<Inner>>,
    Path(path): Path<String>,
    h: HeaderMap,
    b: Bytes,
) -> Response {
    let body: Value = serde_json::from_slice(&b).unwrap();
    record(&i, &path, "POST", h.clone(), HashMap::new(), body.clone());
    assert_eq!(h["authorization"], format!("Bearer {TOKEN}"));
    assert!(body.get("scheduled_publish_time").is_none());
    if let Some(response) = fault(&i, &path).await {
        return response;
    }
    let answer = match path.as_str() {
        "101/feed" => json!({"id":"101_404"}),
        "111/feed" => json!({"id":"111_404"}),
        "101/photos" => {
            if i.config.lock().unwrap().photo_only {
                json!({"id":"505"})
            } else {
                json!({"id":"505","post_id":"101_404"})
            }
        }
        "202/media" | "212/media" => {
            let mut ids = i.containers.lock().unwrap();
            let id = (600 + ids.len()).to_string();
            ids.push(id.clone());
            json!({"id":id})
        }
        "202/media_publish" | "212/media_publish" => {
            assert!(
                i.containers
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|id| Some(id.as_str()) == body["creation_id"].as_str())
            );
            json!({"id":"303"})
        }
        _ => panic!("unexpected Graph path: {path}"),
    };
    axum::Json(answer).into_response()
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
fn record(
    i: &Inner,
    path: &str,
    method: &str,
    headers: HeaderMap,
    query: HashMap<String, String>,
    body: Value,
) {
    i.calls.lock().unwrap().push(Call {
        path: path.into(),
        method: method.into(),
        headers,
        query,
        body,
    });
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn fault(i: &Inner, path: &str) -> Option<Response> {
    let c = i.config.lock().unwrap().clone();
    if c.fault_path.as_deref() != Some(path)
        || i.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.path == path)
            .count()
            <= c.fault_after
    {
        return None;
    }
    if c.hang {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    if c.empty_body {
        return Some(
            StatusCode::from_u16(c.fault_status)
                .unwrap()
                .into_response(),
        );
    }
    if c.malformed {
        return Some(axum::Json(json!({"unexpected":true})).into_response());
    }
    Some(
        (
            StatusCode::from_u16(c.fault_status).unwrap(),
            axum::Json(json!({"error":{"code":c.fault_code,"message":"synthetic error"}})),
        )
            .into_response(),
    )
}
