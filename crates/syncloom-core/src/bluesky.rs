//! Bluesky via maintained ATrium API 0.26.0, with explicit session renewal.
use crate::{
    BaseUrl, Clock, Platform, PostFailure, PostFuture, PostRequest, Posted, Secret, StudioError,
    TokenStore,
};
use crate::{
    bluesky_transport::{Transport, failure},
    credential_session::CredentialSession,
    oauth::Connection,
    social::{account, decode, encode},
};
use atrium_api::{
    client::AtpServiceClient,
    com::atproto::{repo::create_record, server::create_session},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct Bundle {
    password: Secret,
    session: create_session::Output,
}
pub struct Bluesky {
    base: BaseUrl,
    pub(crate) credentials: CredentialSession,
}
impl Bluesky {
    pub(crate) fn new(base: BaseUrl, store: Arc<dyn TokenStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            credentials: CredentialSession::new(
                account("bluesky", base.as_str(), ""),
                store,
                clock,
            ),
            base,
        }
    }
    pub fn connection(&self) -> Result<Connection, StudioError> {
        self.credentials.connection(false)
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.credentials.clear()
    }
    pub async fn sign_in(
        &self,
        identifier: &str,
        password: Secret,
    ) -> Result<Connection, StudioError> {
        if identifier.is_empty() || identifier.contains('@') || password.expose().trim().is_empty()
        {
            return Err(StudioError::InvalidRequest);
        }
        let _guard = self.credentials.auth.lock().await;
        self.credentials.invalidate();
        let generation = self.credentials.generation();
        let api = AtpServiceClient::new(Transport::new(self.base.clone(), None));
        let session = api
            .service
            .com
            .atproto
            .server
            .create_session(
                create_session::InputData {
                    identifier: identifier.into(),
                    password: password.expose().into(),
                    allow_takendown: None,
                    auth_factor_token: None,
                }
                .into(),
            )
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        self.save(Bundle { password, session }, generation)?;
        self.connection()
    }
    pub async fn refresh(&self) -> Result<Connection, StudioError> {
        let renewal = self
            .credentials
            .renewal(crate::credential_session::RefreshRule::Any)
            .await?;
        let token = &renewal.token;
        let mut bundle: Bundle = decode(token)?;
        let api = AtpServiceClient::new(Transport::new(
            self.base.clone(),
            Some(Secret::new(&bundle.session.refresh_jwt)),
        ));
        let renewed = api
            .service
            .com
            .atproto
            .server
            .refresh_session()
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        self.credentials
            .confirm_identity(Some(bundle.session.did.as_str()), renewed.did.as_str())?;
        bundle.session.access_jwt = renewed.access_jwt.clone();
        bundle.session.refresh_jwt = renewed.refresh_jwt.clone();
        self.save(bundle, renewal.generation)?;
        self.connection()
    }
    fn save(&self, bundle: Bundle, generation: u64) -> Result<(), StudioError> {
        let expires_at =
            jwt_expiry(&bundle.session.access_jwt).ok_or(StudioError::SignInRejected)?;
        if expires_at <= self.credentials.clock.now_rfc3339()
            || bundle.session.refresh_jwt.is_empty()
        {
            return Err(StudioError::SignInRejected);
        }
        self.credentials.commit(
            generation,
            encode(&bundle)?,
            expires_at,
            bundle.session.did.to_string(),
            "app-password".into(),
        )
    }
}
impl Platform for Bluesky {
    fn posting_target(&self) -> Result<String, PostFailure> {
        let token = self.credentials.usable()?;
        crate::platform::account_target("bluesky", "", &token.member_id)
    }
    fn publish(&self, request: PostRequest) -> PostFuture<'_> {
        Box::pin(async move {
            crate::social::check_request(crate::Channel::Bluesky, &request, 1_000_000)?;
            let token = self.credentials.usable()?;
            let bundle: Bundle = decode(&token).map_err(|_| PostFailure::ConsentRequired)?;
            if bundle.session.did.to_string() != token.member_id {
                return Err(PostFailure::ConsentRequired);
            }
            request.require_target(crate::platform::account_target(
                "bluesky",
                "",
                &token.member_id,
            ))?;
            let api = AtpServiceClient::new(Transport::new(
                self.base.clone(),
                Some(Secret::new(bundle.session.access_jwt.clone())),
            ));
            let mut record = serde_json::json!({"$type":"app.bsky.feed.post","text":request.text,"createdAt":self.credentials.clock.now_rfc3339()});
            if let Some(image) = request.image {
                let blob = api
                    .service
                    .com
                    .atproto
                    .repo
                    .upload_blob(image.bytes)
                    .await
                    .map_err(failure)?;
                record["embed"] = serde_json::json!({"$type":"app.bsky.embed.images","images":[{"alt":image.alt_text,"image":blob.blob}]});
            }
            let input:create_record::Input=serde_json::from_value(serde_json::json!({"repo":token.member_id,"collection":"app.bsky.feed.post","record":record,"rkey":request.request_id})).map_err(|_|PostFailure::Refused)?;
            let output = api
                .service
                .com
                .atproto
                .repo
                .create_record(input)
                .await
                .map_err(failure)?;
            receipt(&token.member_id, &output.uri)
        })
    }
}
fn receipt(member: &str, uri: &str) -> Result<Posted, PostFailure> {
    let prefix = format!("at://{member}/app.bsky.feed.post/");
    let key = uri
        .strip_prefix(&prefix)
        .filter(|s| {
            !s.is_empty()
                && !s.contains('/')
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        .ok_or(PostFailure::Unknown)?;
    Ok(Posted {
        url: format!("https://bsky.app/profile/{member}/post/{key}"),
        post_id: uri.into(),
    })
}
fn jwt_expiry(jwt: &str) -> Option<String> {
    use base64::Engine;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(jwt.split('.').nth(1)?)
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    time::OffsetDateTime::from_unix_timestamp(value.get("exp")?.as_i64()?)
        .ok()
        .map(crate::clock::rfc3339)
}
