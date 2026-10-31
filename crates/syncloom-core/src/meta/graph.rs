use super::{GRAPH_VERSION, Meta};
use crate::{
    PostFailure, Secret,
    platform::{http_failure, transport_failure},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
impl Meta {
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}/{GRAPH_VERSION}/{path}", self.config.graph_base.as_str())
    }
    pub(super) async fn json<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, PostFailure> {
        let status = response.status();
        let body: Value = crate::oauth::read_json_body(response)
            .await
            .ok_or_else(|| http_failure(status).unwrap_or(PostFailure::Unknown))?;
        // Graph uses 400 for invalidated tokens and throttling. Do not classify solely by HTTP.
        if let Some(error) = body.get("error") {
            return Err(match error["code"].as_u64() {
                Some(190 | 102) => PostFailure::ConsentRequired,
                Some(4 | 17 | 32 | 613 | 80004) => PostFailure::RateLimited,
                _ => http_failure(status).unwrap_or(PostFailure::Unknown),
            });
        }
        if let Some(error) = http_failure(status) {
            return Err(error);
        }
        serde_json::from_value(body).map_err(|_| PostFailure::Unknown)
    }
    pub(super) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        token: &Secret,
        fields: &[(&str, &str)],
    ) -> Result<T, PostFailure> {
        self.json(
            self.client
                .get(self.url(path))
                .bearer_auth(token.expose())
                .query(fields)
                .send()
                .await
                .map_err(transport_failure)?,
        )
        .await
    }
    pub(super) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        token: &Secret,
        body: Value,
    ) -> Result<T, PostFailure> {
        self.json(
            self.client
                .post(self.url(path))
                .bearer_auth(token.expose())
                .json(&body)
                .send()
                .await
                .map_err(transport_failure)?,
        )
        .await
    }
}
