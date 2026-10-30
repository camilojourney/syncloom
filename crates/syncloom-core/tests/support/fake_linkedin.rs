//! A fake LinkedIn on loopback: the authorization pages, the token endpoint, `userinfo` and
//! the Posts API, with scripted answers and recorded requests. Both base URLs (auth and API)
//! are served on one address. Included by the core tests, the CLI tests and the
//! `fake_linkedin` example. It never contacts linkedin.com.
#![allow(dead_code)]
use axum::{
    Router,
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Notify;

pub const MEMBER_ID: &str = "FAKEMEMBER01";
pub const DEFAULT_TOKEN: &str = "fake-access-token-zebrafish";
pub const DEFAULT_POST_ID: &str = "urn:li:share:7000000000000000001";

/// How the authorization page answers the browser.
#[derive(Clone, Debug)]
pub enum AuthMode {
    /// Redirect back with a code and the state that was sent.
    Consent,
    /// Redirect back with `error=user_cancelled_authorize` and the state.
    Deny,
    /// Redirect back with a code and a different state.
    WrongState,
}

/// How the token endpoint answers.
#[derive(Clone, Debug)]
pub enum TokenMode {
    /// Issue this token, valid for `expires_in` seconds, with this scope.
    Ok {
        access_token: String,
        expires_in: u64,
        scope: String,
    },
    /// Answer this status with a JSON error body.
    Status(u16),
}

impl TokenMode {
    pub fn ok() -> TokenMode {
        TokenMode::Ok {
            access_token: DEFAULT_TOKEN.to_owned(),
            expires_in: 5_184_000,
            scope: "openid,profile,w_member_social".to_owned(),
        }
    }
}

/// How `POST /rest/posts` answers.
#[derive(Clone, Debug)]
pub enum PostMode {
    /// 201 with this `x-restli-id`.
    Posted(String),
    /// 201 without an id header.
    NoId,
    /// This status with a JSON error body.
    Status(u16),
    /// Wait a long time before answering 201.
    Slow,
    /// Wait until [`FakeLinkedin::release`] is called, then answer 201 with the default id.
    Hold,
}

impl PostMode {
    /// Parse the example's `--post-mode` value.
    pub fn parse(value: &str) -> Result<PostMode, String> {
        Ok(match value {
            "posted" => PostMode::Posted(DEFAULT_POST_ID.to_owned()),
            "noid" => PostMode::NoId,
            "slow" => PostMode::Slow,
            status => PostMode::Status(
                status
                    .parse()
                    .map_err(|_| format!("unknown post mode {value}"))?,
            ),
        })
    }
}

/// One authorization request: its query parameters, as the browser sent them.
#[derive(Clone, Debug)]
pub struct AuthRequest {
    pub path: String,
    pub query: HashMap<String, String>,
}

/// One token request: the form fields and whether an Authorization header came with it.
#[derive(Clone, Debug)]
pub struct TokenRequest {
    pub form: HashMap<String, String>,
    pub authorization: Option<String>,
}

/// One Posts API request: every header (lower-case names) and the JSON body.
#[derive(Clone, Debug)]
pub struct PostRecorded {
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub raw_body: String,
}

#[derive(Clone)]
struct Shared {
    auth: Arc<Mutex<AuthMode>>,
    token: Arc<Mutex<TokenMode>>,
    post: Arc<Mutex<PostMode>>,
    gate: Arc<Notify>,
    auth_requests: Arc<Mutex<Vec<AuthRequest>>>,
    token_requests: Arc<Mutex<Vec<TokenRequest>>>,
    userinfo_bearers: Arc<Mutex<Vec<Option<String>>>>,
    posts: Arc<Mutex<Vec<PostRecorded>>>,
    codes_issued: Arc<Mutex<u32>>,
}

pub struct FakeLinkedin {
    pub addr: SocketAddr,
    shared: Shared,
}

impl FakeLinkedin {
    /// Serves both the auth base and the API base.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
    pub fn set_auth(&self, mode: AuthMode) {
        *self.shared.auth.lock().unwrap() = mode;
    }
    pub fn set_token(&self, mode: TokenMode) {
        *self.shared.token.lock().unwrap() = mode;
    }
    pub fn set_post(&self, mode: PostMode) {
        *self.shared.post.lock().unwrap() = mode;
    }
    /// Let a held `POST /rest/posts` answer.
    pub fn release(&self) {
        self.shared.gate.notify_one();
    }
    pub fn auth_requests(&self) -> Vec<AuthRequest> {
        self.shared.auth_requests.lock().unwrap().clone()
    }
    pub fn token_requests(&self) -> Vec<TokenRequest> {
        self.shared.token_requests.lock().unwrap().clone()
    }
    pub fn userinfo_bearers(&self) -> Vec<Option<String>> {
        self.shared.userinfo_bearers.lock().unwrap().clone()
    }
    pub fn posts(&self) -> Vec<PostRecorded> {
        self.shared.posts.lock().unwrap().clone()
    }
}

pub async fn start(auth: AuthMode, token: TokenMode, post_mode: PostMode) -> FakeLinkedin {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shared = Shared {
        auth: Arc::new(Mutex::new(auth)),
        token: Arc::new(Mutex::new(token)),
        post: Arc::new(Mutex::new(post_mode)),
        gate: Arc::new(Notify::new()),
        auth_requests: Arc::new(Mutex::new(Vec::new())),
        token_requests: Arc::new(Mutex::new(Vec::new())),
        userinfo_bearers: Arc::new(Mutex::new(Vec::new())),
        posts: Arc::new(Mutex::new(Vec::new())),
        codes_issued: Arc::new(Mutex::new(0)),
    };
    let app = Router::new()
        .route("/oauth/v2/authorization", get(authorize))
        .route("/oauth/native-pkce/authorization", get(authorize))
        .route("/oauth/v2/accessToken", post(access_token))
        .route("/v2/userinfo", get(userinfo))
        .route("/rest/posts", post(posts))
        .with_state(shared.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    FakeLinkedin { addr, shared }
}

/// The browser arrives here; the fake plays the member and redirects straight back.
async fn authorize(
    State(shared): State<Shared>,
    uri: axum::http::Uri,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    shared.auth_requests.lock().unwrap().push(AuthRequest {
        path: uri.path().to_owned(),
        query: query.clone(),
    });
    let Some(redirect) = query.get("redirect_uri") else {
        return (StatusCode::BAD_REQUEST, "missing redirect_uri").into_response();
    };
    let state = query.get("state").cloned().unwrap_or_default();
    let mode = shared.auth.lock().unwrap().clone();
    let code = {
        let mut issued = shared.codes_issued.lock().unwrap();
        *issued += 1;
        format!("fake-code-{}", *issued)
    };
    let target = match mode {
        AuthMode::Consent => format!("{redirect}?code={code}&state={state}"),
        AuthMode::Deny => format!(
            "{redirect}?error=user_cancelled_authorize&error_description=zebrafish%20said%20no&state={state}"
        ),
        AuthMode::WrongState => format!("{redirect}?code={code}&state=not-the-state"),
    };
    Redirect::temporary(&target).into_response()
}

async fn access_token(State(shared): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
    let form: HashMap<String, String> = form_fields(&String::from_utf8_lossy(&body));
    shared.token_requests.lock().unwrap().push(TokenRequest {
        form,
        authorization: header_string(&headers, "authorization"),
    });
    match shared.token.lock().unwrap().clone() {
        TokenMode::Ok {
            access_token,
            expires_in,
            scope,
        } => axum::Json(serde_json::json!({
            "access_token": access_token,
            "expires_in": expires_in,
            "scope": scope,
            "token_type": "Bearer",
            "id_token": "fake.id.token"
        }))
        .into_response(),
        TokenMode::Status(status) => error_body(status),
    }
}

async fn userinfo(State(shared): State<Shared>, headers: HeaderMap) -> Response {
    let bearer = header_string(&headers, "authorization")
        .and_then(|value| value.strip_prefix("Bearer ").map(str::to_owned));
    shared.userinfo_bearers.lock().unwrap().push(bearer.clone());
    if bearer.is_none() {
        return error_body(401);
    }
    axum::Json(serde_json::json!({
        "sub": MEMBER_ID,
        "name": "Fake Member",
        "locale": {"country": "US", "language": "en"}
    }))
    .into_response()
}

async fn posts(State(shared): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
    let raw_body = String::from_utf8_lossy(&body).into_owned();
    shared.posts.lock().unwrap().push(PostRecorded {
        headers: headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|v| (name.as_str().to_owned(), v.to_owned()))
            })
            .collect(),
        body: serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
        raw_body,
    });
    let mode = shared.post.lock().unwrap().clone();
    match mode {
        PostMode::Posted(id) => created(Some(&id)),
        PostMode::NoId => created(None),
        PostMode::Status(status) => error_body(status),
        PostMode::Slow => {
            tokio::time::sleep(Duration::from_secs(30)).await;
            created(Some(DEFAULT_POST_ID))
        }
        PostMode::Hold => {
            shared.gate.notified().await;
            created(Some(DEFAULT_POST_ID))
        }
    }
}

fn created(id: Option<&str>) -> Response {
    let mut response = StatusCode::CREATED.into_response();
    if let Some(id) = id {
        response
            .headers_mut()
            .insert("x-restli-id", id.parse().unwrap());
    }
    response
}

fn error_body(status: u16) -> Response {
    (
        StatusCode::from_u16(status).unwrap(),
        axum::Json(serde_json::json!({
            "status": status,
            "serviceErrorCode": 65600,
            "message": "fake linkedin error body zebrafish",
            "code": "FAKE"
        })),
    )
        .into_response()
}

/// `a=b&c=d` with percent-decoding and `+` as space, as a token endpoint reads it.
fn form_fields(body: &str) -> HashMap<String, String> {
    body.split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(name), percent_decode(value))
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            byte => out.push(byte),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .or_else(|| headers.get(header::HeaderName::from_bytes(name.as_bytes()).ok()?))
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}
