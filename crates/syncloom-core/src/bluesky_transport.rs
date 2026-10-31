//! Typed ATrium SDK over the studio's bounded, no-redirect transport. No DID URL discovery.
use crate::{BaseUrl, PostFailure, Secret};
use atrium_api::xrpc::{
    HttpClient, XrpcClient,
    http::{Request, Response},
};

pub(crate) struct Transport {
    base: BaseUrl,
    token: Option<Secret>,
    client: reqwest::Client,
}
impl Transport {
    pub fn new(base: BaseUrl, token: Option<Secret>) -> Self {
        Self {
            base,
            token,
            client: crate::config::http_client(),
        }
    }
}
impl XrpcClient for Transport {
    fn base_uri(&self) -> String {
        self.base.as_str().into()
    }
}
impl HttpClient for Transport {
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let (mut parts, body) = request.into_parts();
        // Replace, rather than append: ATrium supplies the Lexicon wildcard already.
        if parts.uri.path().ends_with("/com.atproto.repo.uploadBlob") {
            parts.headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("image/png"),
            );
        }
        let mut builder = self
            .client
            .request(parts.method, parts.uri.to_string())
            .headers(parts.headers)
            .body(body);
        if let Some(token) = &self.token {
            builder = builder.bearer_auth(token.expose());
        }
        let mut response = builder.send().await?;
        let mut result = Response::builder().status(response.status());
        *result.headers_mut().expect("response builder") = response.headers().clone();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 64 * 1024 {
                return Err(std::io::Error::other("bounded platform response").into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(result.body(bytes)?)
    }
}
pub(crate) fn failure<E: std::fmt::Debug>(error: atrium_api::xrpc::Error<E>) -> PostFailure {
    use atrium_api::xrpc::{Error, error::XrpcErrorKind};
    match error {
        Error::XrpcResponse(response) => {
            if response.status.is_server_error() {
                return PostFailure::Unknown;
            }
            if matches!(&response.error,Some(XrpcErrorKind::Undefined(e)) if matches!(e.error.as_deref(),Some("ExpiredToken"|"InvalidToken")))
            {
                return PostFailure::ConsentRequired;
            }
            crate::platform::http_failure(response.status).unwrap_or(PostFailure::Unknown)
        }
        Error::HttpClient(error)
            if error
                .downcast_ref::<reqwest::Error>()
                .is_some_and(|e| e.is_connect()) =>
        {
            PostFailure::Unreachable
        }
        _ => PostFailure::Unknown,
    }
}
