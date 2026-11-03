//! One web access policy for every route, including static files and unknown paths.
use crate::{
    failure::Failure,
    server::{AppState, read_bounded},
};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use syncloom_core::Secret;

pub(crate) struct Boundary {
    host: String,
    token: Secret,
}

fn single<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let mut values = headers.get_all(name).iter();
    match (values.next(), values.next()) {
        (Some(value), None) => value.to_str().ok(),
        _ => None,
    }
}

impl Boundary {
    pub(crate) fn new(host: String) -> Self {
        Self {
            host,
            token: Secret::new(format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            )),
        }
    }
    pub(crate) fn token(&self) -> &str {
        self.token.expose()
    }

    fn headers(&self, headers: &HeaderMap) -> Result<(), Failure> {
        if single(headers, "host") != Some(self.host.as_str()) {
            return Err(Failure::HostNotAllowed);
        }
        if headers.contains_key(header::ORIGIN)
            && single(headers, "origin") != Some(format!("http://{}", self.host).as_str())
        {
            return Err(Failure::OriginNotAllowed);
        }
        if headers.contains_key("sec-fetch-site")
            && !matches!(
                single(headers, "sec-fetch-site"),
                Some("same-origin" | "none")
            )
        {
            return Err(Failure::OriginNotAllowed);
        }
        Ok(())
    }

    async fn authorize(&self, request: Request) -> Result<Request, Failure> {
        self.headers(request.headers())?;
        if matches!(
            *request.method(),
            Method::GET | Method::HEAD | Method::OPTIONS
        ) {
            return Ok(request);
        }
        if is_page_path(request.uri().path()) {
            // Read a bounded form once, then restore it for the typed form adapter.
            let (parts, body) = request.into_parts();
            let bytes = read_bounded(body).await?;
            let tokens: Vec<_> = url::form_urlencoded::parse(&bytes)
                .filter(|(name, _)| name == "csrf")
                .map(|(_, value)| value)
                .collect();
            if tokens.len() != 1 || !self.token.matches(&tokens[0]) {
                return Err(Failure::SessionRequired);
            }
            Ok(Request::from_parts(parts, Body::from(bytes)))
        } else if single(request.headers(), "x-syncloom-token")
            .is_some_and(|token| self.token.matches(token))
        {
            Ok(request)
        } else {
            Err(Failure::SessionRequired)
        }
    }
}

pub(crate) fn is_page_path(path: &str) -> bool {
    path == "/"
        || path == "/capture"
        || path.starts_with("/capture/")
        || path.starts_with("/review/")
        || path.starts_with("/actions/")
}

pub(crate) async fn middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    match state.boundary.authorize(request).await {
        Ok(request) => next.run(request).await,
        Err(failure) => failure.into_response(),
    }
}

/// Applied after error rendering so even refused requests carry the same protections.
#[expect(
    clippy::expect_used,
    reason = "Security header values contain only fixed ASCII literals, never request data."
)]
pub(crate) fn protect(response: &mut Response) {
    // Only verified PDF bytes may be framed by our own studio. Pages remain unframeable.
    let pdf = response
        .headers()
        .get("content-type")
        .is_some_and(|v| v == "application/pdf");
    let ancestors = if pdf { "'self'" } else { "'none'" };
    let csp = format!(
        "default-src 'none'; script-src 'none'; style-src 'self'; img-src 'self'; frame-src 'self'; object-src 'none'; connect-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors {ancestors}"
    );
    for (name, value) in [
        ("cache-control", "no-store"),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "same-origin"),
        ("x-frame-options", if pdf { "SAMEORIGIN" } else { "DENY" }),
        ("content-security-policy", csp.as_str()),
    ] {
        response
            .headers_mut()
            .insert(name, value.parse().expect("fixed ASCII security header"));
    }
}
