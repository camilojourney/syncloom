//! Independent loopback servers with only one platform's routes enabled per instance.
#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
use axum::{
    Router,
    body::to_bytes,
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct Call {
    pub target_account: Option<String>,
    pub method: String,
    pub path: String,
    pub query: String,
    pub authorization: String,
    pub content_type: String,
    pub body: Vec<u8>,
}
impl Call {
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
    pub fn form(&self) -> HashMap<String, String> {
        url::form_urlencoded::parse(&self.body)
            .into_owned()
            .collect()
    }
}
#[derive(Default)]
struct Data {
    calls: Vec<Call>,
    principals: HashMap<String, String>,
    replies: HashMap<String, VecDeque<(u16, Value)>>,
    holds: HashMap<String, Arc<tokio::sync::Notify>>,
    changed: Arc<tokio::sync::Notify>,
}
pub struct Fake {
    base: String,
    data: Arc<Mutex<Data>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Fake {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Fake {
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    async fn start(routes: Vec<(&str, Value)>) -> Self {
        let data = Arc::new(Mutex::new(Data::default()));
        for (path, body) in routes {
            data.lock()
                .unwrap()
                .replies
                .insert(path.into(), VecDeque::from([(200, body)]));
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let app = Router::new().fallback(handle).with_state(data.clone());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self { base, data, task }
    }
    pub fn base(&self) -> &str {
        &self.base
    }
    pub fn principal(&self, access: &str, id: &str) {
        assert!(
            self.data
                .lock()
                .map(|mut data| {
                    data.principals
                        .insert(format!("Bearer {access}"), id.into());
                })
                .is_ok()
        );
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn calls(&self) -> Vec<Call> {
        self.data.lock().unwrap().calls.clone()
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn reply(&self, path: &str, status: u16, body: Value) {
        self.data
            .lock()
            .unwrap()
            .replies
            .insert(path.into(), VecDeque::from([(status, body)]));
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn sequence(&self, path: &str, replies: Vec<(u16, Value)>) {
        self.data
            .lock()
            .unwrap()
            .replies
            .insert(path.into(), replies.into());
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn hold(&self, path: &str) -> Arc<tokio::sync::Notify> {
        let gate = Arc::new(tokio::sync::Notify::new());
        self.data
            .lock()
            .unwrap()
            .holds
            .insert(path.into(), gate.clone());
        gate
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub async fn wait_calls(&self, count: usize) {
        let changed = self.data.lock().unwrap().changed.clone();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let notified = changed.notified();
                if self.calls().len() >= count {
                    break;
                }
                notified.await;
            }
        })
        .await
        .unwrap();
    }
    pub async fn bluesky() -> Self {
        Self::start(vec![
            ("/xrpc/com.atproto.server.createSession",json!({"accessJwt":jwt(1793455200),"refreshJwt":"refresh-bsky-1","did":"did:plc:testrepo","handle":"test.invalid"})),
            ("/xrpc/com.atproto.server.refreshSession",json!({"accessJwt":jwt(1793541600),"refreshJwt":"refresh-bsky-2","did":"did:plc:testrepo","handle":"test.invalid"})),
            ("/xrpc/com.atproto.repo.uploadBlob",json!({"blob":{"$type":"blob","ref":{"$link":CID},"mimeType":"image/png","size":8}})),
            ("/xrpc/com.atproto.repo.createRecord",json!({"uri":"at://did:plc:testrepo/app.bsky.feed.post/testrecord","cid":CID}))
        ]).await
    }
    pub async fn x() -> Self {
        Self::start(vec![
            ("/2/users/me",json!({"data":{"id":"111"}})),
            ("/2/oauth2/token",json!({"access_token":"x-access-1","refresh_token":"x-refresh-1","expires_in":7200,"token_type":"bearer","scope":"tweet.read tweet.write users.read media.write offline.access"})),
            ("/2/media/upload/initialize",json!({"data":{"id":"101"}})),
            ("/2/media/upload/101/append",json!({})),
            ("/2/media/upload/101/finalize",json!({"data":{"id":"101"}})),
            ("/2/media/metadata",json!({"data":{}})),
            ("/2/tweets",json!({"data":{"id":"202","text":"Test"}}))
        ]).await
    }
    pub async fn threads() -> Self {
        Self::start(vec![
            (
                "/oauth/access_token",
                json!({"access_token":"threads-short","user_id":303}),
            ),
            (
                "/access_token",
                json!({"access_token":"threads-long-1","expires_in":5184000,"token_type":"bearer"}),
            ),
            (
                "/refresh_access_token",
                json!({"access_token":"threads-long-2","expires_in":5184000,"token_type":"bearer"}),
            ),
            ("/v1.0/me", json!({"id":"303"})),
            ("/v1.0/303/threads", json!({"id":"404"})),
            ("/v1.0/404", json!({"id":"404","status":"FINISHED"})),
            ("/v1.0/303/threads_publish", json!({"id":"505"})),
            (
                "/v1.0/505",
                json!({"id":"505","permalink":"https://www.threads.com/@test/post/abc"}),
            ),
        ])
        .await
    }
}
pub const CID: &str = "bafyreie5cvhasmsq7xfp7rldv6gp34t65sdrhj3xn7mkkepd7q2slb4z3i";
pub fn jwt(exp: i64) -> String {
    use base64::Engine;
    format!(
        "e30.{}.test",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!("{{\"exp\":{exp}}}"))
    )
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn handle(State(data): State<Arc<Mutex<Data>>>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    let call = Call {
        target_account: None,
        method: request.method().to_string(),
        path: path.clone(),
        query: request.uri().query().unwrap_or("").into(),
        authorization: request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .into(),
        content_type: request
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .into(),
        body: Vec::new(),
    };
    let bytes = to_bytes(request.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    let hold = {
        let data = data.lock().unwrap();
        data.holds.get(&path).cloned()
    };
    let (status, body) = {
        let mut data = data.lock().unwrap();
        let target_account = data.principals.get(&call.authorization).cloned();
        data.calls.push(Call {
            target_account,
            body: bytes.to_vec(),
            ..call
        });
        data.changed.notify_one();
        match data.replies.get_mut(&path) {
            Some(queue) if queue.len() > 1 => queue.pop_front().unwrap(),
            Some(queue) => queue.front().unwrap().clone(),
            None => (404, json!({"error":"UnexpectedEndpoint"})),
        }
    };
    if let Some(hold) = hold {
        hold.notified().await;
    }
    if status == 599 {
        std::future::pending::<()>().await;
    }
    (
        axum::http::StatusCode::from_u16(status.min(599)).unwrap(),
        axum::Json(body),
    )
        .into_response()
}
