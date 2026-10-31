//! Stateful loopback Mastodon boundary. No endpoint can contact a real instance.
use axum::{
    Router,
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub const TOKEN: &str = "fake-mastodon-token-zebrafish";
#[derive(Clone)]
pub struct Config {
    pub status: u16,
    pub max_chars: usize,
    pub max_image_bytes: usize,
    pub async_media: bool,
    pub media_status: u16,
    pub wrong_state: bool,
    pub deny: bool,
    pub no_id: bool,
    pub hang_status: bool,
    pub pending_forever: bool,
    pub token_status: u16,
    pub bad_limits: bool,
    pub oversized: bool,
    pub media_id: String,
    pub foreign_url: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            status: 200,
            max_chars: 500,
            max_image_bytes: 10_000_000,
            async_media: false,
            media_status: 200,
            wrong_state: false,
            deny: false,
            no_id: false,
            hang_status: false,
            pending_forever: false,
            token_status: 200,
            bad_limits: false,
            oversized: false,
            media_id: "media-1".into(),
            foreign_url: false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Call {
    pub path: &'static str,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}
struct Inner {
    config: Config,
    calls: Mutex<Vec<Call>>,
    base: String,
    polls: Mutex<usize>,
}
pub struct FakeMastodon {
    inner: Arc<Inner>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for FakeMastodon {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl FakeMastodon {
    pub fn base(&self) -> &str {
        &self.inner.base
    }
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
}
fn record(i: &Inner, path: &'static str, h: HeaderMap, b: Vec<u8>) {
    i.calls.lock().unwrap().push(Call {
        path,
        headers: h,
        body: b,
    });
}
async fn apps(State(i): State<Arc<Inner>>, h: HeaderMap, b: Bytes) -> Response {
    record(&i, "apps", h, b.to_vec());
    axum::Json(json!({"client_id":"fake-app","client_secret":"fake-secret-axolotl"}))
        .into_response()
}
async fn authorize(
    State(i): State<Arc<Inner>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let mut url = reqwest::Url::parse(&q["redirect_uri"]).unwrap();
    url.query_pairs_mut().append_pair(
        "state",
        if i.config.wrong_state {
            "wrong"
        } else {
            &q["state"]
        },
    );
    if i.config.deny {
        url.query_pairs_mut().append_pair("error", "access_denied");
    } else {
        url.query_pairs_mut().append_pair("code", "fake-code");
    }
    Redirect::temporary(url.as_str()).into_response()
}
async fn token(State(i): State<Arc<Inner>>, h: HeaderMap, b: Bytes) -> Response {
    record(&i, "token", h, b.to_vec());
    if i.config.token_status != 200 {
        return StatusCode::from_u16(i.config.token_status)
            .unwrap()
            .into_response();
    }
    axum::Json(json!({"access_token":TOKEN,"token_type":"Bearer","scope":"write:statuses write:media","created_at":1793383200})).into_response()
}
async fn instance(State(i): State<Arc<Inner>>, h: HeaderMap) -> Response {
    record(&i, "instance", h, vec![]);
    if i.config.bad_limits {
        return axum::Json(json!({"configuration":{}})).into_response();
    }
    let mut value = json!({"configuration":{"statuses":{"max_characters":i.config.max_chars,"max_media_attachments":4},"media_attachments":{"image_size_limit":i.config.max_image_bytes,"supported_mime_types":["image/png"]}}});
    if i.config.oversized {
        value["padding"] = json!("x".repeat(70_000));
    }
    axum::Json(value).into_response()
}
async fn media(State(i): State<Arc<Inner>>, h: HeaderMap, b: Bytes) -> Response {
    record(&i, "media", h, b.to_vec());
    if i.config.media_status != 200 {
        return StatusCode::from_u16(i.config.media_status)
            .unwrap()
            .into_response();
    }
    let status = if i.config.async_media {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };
    (status,axum::Json(json!({"id":i.config.media_id,"url":if i.config.async_media {Value::Null} else {json!(format!("{}/media/1",i.base))}}))).into_response()
}
async fn poll(State(i): State<Arc<Inner>>, h: HeaderMap) -> Response {
    record(&i, "poll", h, vec![]);
    let mut count = i.polls.lock().unwrap();
    *count += 1;
    if i.config.pending_forever || *count == 1 {
        return StatusCode::PARTIAL_CONTENT.into_response();
    }
    axum::Json(json!({"id":"media-1","url":format!("{}/media/1",i.base)})).into_response()
}
async fn statuses(State(i): State<Arc<Inner>>, h: HeaderMap, b: Bytes) -> Response {
    record(&i, "statuses", h, b.to_vec());
    if i.config.hang_status {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    let status = StatusCode::from_u16(i.config.status).unwrap();
    (
        status,
        axum::Json(if i.config.no_id {
            json!({"id":"","url":format!("{}/@fixture/1",i.base)})
        } else {
            json!({"id":"status-1","url":if i.config.foreign_url { "https://elsewhere.invalid/1".to_owned() } else { format!("{}/@fixture/1",i.base) }})
        }),
    )
        .into_response()
}
pub async fn start(config: Config) -> FakeMastodon {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let inner = Arc::new(Inner {
        config,
        calls: Mutex::new(vec![]),
        base,
        polls: Mutex::new(0),
    });
    let expected = listener.local_addr().unwrap().to_string();
    let app = Router::new()
        .route("/api/v1/apps", post(apps))
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/token", post(token))
        .route("/api/v2/instance", get(instance))
        .route("/api/v2/media", post(media))
        .route("/api/v1/media/media-1", get(poll))
        .route("/api/v1/statuses", post(statuses))
        .layer(axum::middleware::from_fn(move |request: axum::extract::Request, next: axum::middleware::Next| {
            let expected = expected.clone();
            async move {
                let mut hosts = request.headers().get_all("host").iter();
                if matches!((hosts.next(),hosts.next()),(Some(h),None) if h.as_bytes()==expected.as_bytes()) { next.run(request).await }
                else { (StatusCode::FORBIDDEN, axum::Json(json!({"error":{"code":"host_not_allowed","message":"Host refused."}}))).into_response() }
            }
        }))
        .with_state(inner.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    FakeMastodon { inner, task }
}
