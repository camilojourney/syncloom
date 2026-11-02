use super::{GRAPH_VERSION, Meta, SCOPES, config::numeric_id};
use crate::{Connection, Secret, SignInStart, StudioError, Token};
use serde::Deserialize;
use std::collections::HashSet;
impl Meta {
    #[expect(
        clippy::expect_used,
        reason = "BaseUrl validates the authorization base and the graph OAuth path is fixed."
    )]
    pub fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let (client, _, configuration, _) = self.config.require()?;
        let mut url = reqwest::Url::parse(&format!(
            "{}/{GRAPH_VERSION}/dialog/oauth",
            self.config.auth_base.as_str()
        ))
        .expect("checked base");
        url.query_pairs_mut()
            .append_pair("client_id", client)
            .append_pair("redirect_uri", self.config.redirect.as_str())
            .append_pair("config_id", configuration)
            .append_pair("response_type", "code");
        self.session.begin(url, false)
    }
    pub async fn sign_in_wait(&self) -> Result<Connection, StudioError> {
        self.config.require()?;
        let (code, _, generation) = self.session.wait().await?;
        let token = tokio::time::timeout(self.session.deadline(), self.derive_token(&code))
            .await
            .map_err(|_| StudioError::SignInTimedOut)??;
        self.session.save_token(generation, &token)?;
        self.connection()
    }
    async fn derive_token(&self, code: &str) -> Result<Token, StudioError> {
        let (_, _, _, page_id) = self.config.require()?;
        let short = self
            .exchange_user_token(&[
                ("redirect_uri", self.config.redirect.as_str()),
                ("code", code),
            ])
            .await?;
        if short.access_token.expose().is_empty() {
            return Err(StudioError::SignInRejected);
        }
        let long = self
            .exchange_user_token(&[
                ("grant_type", "fb_exchange_token"),
                ("fb_exchange_token", short.access_token.expose()),
            ])
            .await?;
        if long.access_token.expose().is_empty() {
            return Err(StudioError::SignInRejected);
        }
        let permissions: Permissions = self
            .get("me/permissions", &long.access_token, &[])
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        let granted: HashSet<_> = permissions
            .data
            .iter()
            .filter(|p| p.status == "granted")
            .map(|p| p.permission.as_str())
            .collect();
        if !SCOPES.iter().all(|p| granted.contains(p)) {
            return Err(StudioError::SignInRejected);
        }
        let page = self.selected_page(&long.access_token, page_id).await?;
        page.into_token(self.session.now_rfc3339())
    }
    async fn exchange_user_token(&self, params: &[(&str, &str)]) -> Result<Issued, StudioError> {
        let (client, secret, _, _) = self.config.require()?;
        let response = self
            .client
            .get(self.url("oauth/access_token"))
            .query(&[("client_id", client), ("client_secret", secret.expose())])
            .query(params)
            .send()
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        self.json(response)
            .await
            .map_err(|_| StudioError::SignInRejected)
    }
    async fn selected_page(&self, token: &Secret, page_id: &str) -> Result<Page, StudioError> {
        let mut after = String::new();
        for _ in 0..10 {
            let pages: Pages = self
                .get(
                    "me/accounts",
                    token,
                    &[
                        ("fields", "id,access_token,tasks,instagram_business_account"),
                        ("limit", "100"),
                        ("after", &after),
                    ],
                )
                .await
                .map_err(|_| StudioError::SignInRejected)?;
            if let Some(page) = pages.data.into_iter().find(|p| p.id == page_id) {
                return Ok(page);
            }
            // Never follow provider-supplied paging URLs to a different origin.
            let Some(cursor) = pages.paging.and_then(|p| p.cursors).and_then(|c| c.after) else {
                break;
            };
            if cursor == after || cursor.len() > 2048 {
                break;
            }
            after = cursor;
        }
        Err(StudioError::SignInRejected)
    }
    pub fn connection(&self) -> Result<Connection, StudioError> {
        self.configured_session()?.connection()
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.configured_session()?.sign_out()
    }
}
#[derive(Deserialize)]
struct Issued {
    access_token: Secret,
}
#[derive(Deserialize)]
struct Permissions {
    data: Vec<Permission>,
}
#[derive(Deserialize)]
struct Permission {
    permission: String,
    status: String,
}
#[derive(Deserialize)]
struct Pages {
    data: Vec<Page>,
    paging: Option<Paging>,
}
#[derive(Deserialize)]
struct Paging {
    cursors: Option<Cursors>,
}
#[derive(Deserialize)]
struct Cursors {
    after: Option<String>,
}
#[derive(Deserialize)]
struct Page {
    id: String,
    access_token: Secret,
    tasks: Vec<String>,
    instagram_business_account: Option<Account>,
}
impl Page {
    fn into_token(self, obtained_at: String) -> Result<Token, StudioError> {
        if self.access_token.expose().is_empty()
            || !["CREATE_CONTENT", "MANAGE", "MODERATE"]
                .iter()
                .all(|t| self.tasks.iter().any(|p| p == t))
        {
            return Err(StudioError::SignInRejected);
        }
        let instagram = self
            .instagram_business_account
            .map(|a| a.id)
            .unwrap_or_default();
        if !instagram.is_empty() && !numeric_id(&instagram) {
            return Err(StudioError::SignInRejected);
        }
        Ok(Token {
            access_token: self.access_token,
            expires_at: None,
            identity_authority: None,
            page_id: Some(self.id),
            member_id: instagram,
            scope: SCOPES.join(" "),
            obtained_at,
        })
    }
}
#[derive(Deserialize)]
struct Account {
    id: String,
}
