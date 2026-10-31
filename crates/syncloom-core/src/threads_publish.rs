//! Threads wire outcomes and the text-only container/publish sequence.
use crate::{PostFailure, PostRequest, Posted, Token};
use serde::{Deserialize, de::DeserializeOwned};
pub(crate) async fn request<T: DeserializeOwned>(
    builder: reqwest::RequestBuilder,
) -> Result<T, PostFailure> {
    let response = builder
        .send()
        .await
        .map_err(crate::platform::transport_failure)?;
    let status = response.status();
    // A server failure may follow a write. Its body cannot make that outcome retryable.
    if status.is_server_error() || status.is_redirection() {
        return Err(PostFailure::Unknown);
    }
    let value: serde_json::Value = crate::oauth::read_json_body(response)
        .await
        .ok_or_else(|| crate::platform::http_failure(status).unwrap_or(PostFailure::Unknown))?;
    decode(status, value)
}
fn decode<T: DeserializeOwned>(
    status: reqwest::StatusCode,
    value: serde_json::Value,
) -> Result<T, PostFailure> {
    if status.is_success() && value.get("error").is_some() {
        return Err(PostFailure::Unknown);
    }
    match value
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_u64())
    {
        Some(190) => return Err(PostFailure::ConsentRequired),
        Some(4 | 32 | 613) => return Err(PostFailure::RateLimited),
        Some(_) => {
            return Err(crate::platform::http_failure(status).unwrap_or(PostFailure::Refused));
        }
        None => {}
    }
    if let Some(f) = crate::platform::http_failure(status) {
        return Err(f);
    }
    serde_json::from_value(value).map_err(|_| PostFailure::Unknown)
}
pub(crate) async fn publish(
    client: &reqwest::Client,
    base: &str,
    token: &Token,
    post: PostRequest,
) -> Result<Posted, PostFailure> {
    let root = format!("{base}/v1.0");
    let container: Id = request(
        client
            .post(format!("{root}/{}/threads", token.member_id))
            .bearer_auth(token.access_token.expose())
            .form(&[("media_type", "TEXT"), ("text", post.text.as_str())]),
    )
    .await?;
    if !crate::social::numeric_id(&container.id) {
        return Err(PostFailure::Unknown);
    }
    // Text should be ready immediately. Never publish an unready container or hide retries.
    let ready: Ready = request(
        client
            .get(format!("{root}/{}", container.id))
            .bearer_auth(token.access_token.expose())
            .query(&[("fields", "id,status")]),
    )
    .await?;
    if ready.id != container.id || ready.status != "FINISHED" {
        return Err(PostFailure::Unknown);
    }
    let published: Id = request(
        client
            .post(format!("{root}/{}/threads_publish", token.member_id))
            .bearer_auth(token.access_token.expose())
            .form(&[("creation_id", container.id)]),
    )
    .await?;
    if !crate::social::numeric_id(&published.id) {
        return Err(PostFailure::Unknown);
    }
    let result: Permalink = request(
        client
            .get(format!("{root}/{}", published.id))
            .bearer_auth(token.access_token.expose())
            .query(&[("fields", "id,permalink")]),
    )
    .await
    // Publication already succeeded. Even a 401/429 here must not permit a new post.
    .map_err(|_| PostFailure::Unknown)?;
    receipt(result, published.id)
}
fn receipt(result: Permalink, published_id: String) -> Result<Posted, PostFailure> {
    let url = crate::config::plain_url(&result.permalink).ok_or(PostFailure::Unknown)?;
    if result.id != published_id
        || url.scheme() != "https"
        || !matches!(
            url.host_str(),
            Some("www.threads.com" | "threads.com" | "www.threads.net" | "threads.net")
        )
        || url.port().is_some()
    {
        return Err(PostFailure::Unknown);
    }
    Ok(Posted {
        post_id: published_id,
        url: result.permalink,
    })
}
#[derive(Deserialize)]
struct Id {
    id: String,
}
#[derive(Deserialize)]
struct Ready {
    id: String,
    status: String,
}
#[derive(Deserialize)]
struct Permalink {
    id: String,
    permalink: String,
}
