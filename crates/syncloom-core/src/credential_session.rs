//! The SocialPlatforms adapters share credential lifecycle policy, not their wire protocols.
use crate::{
    Clock, PostFailure, Secret, StudioError, Token, TokenStore, credentials::CredentialWrites,
    oauth::Connection,
};
use std::sync::Arc;

pub(crate) struct CredentialSession {
    account: String,
    store: Arc<dyn TokenStore>,
    pub clock: Arc<dyn Clock>,
    pub auth: tokio::sync::Mutex<()>,
    writes: CredentialWrites,
}
pub(crate) enum RefreshRule {
    Any,
    UnexpiredAfter(time::Duration),
}
pub(crate) struct Renewal<'a> {
    _guard: tokio::sync::MutexGuard<'a, ()>,
    pub generation: u64,
    pub token: Token,
}
impl CredentialSession {
    pub async fn renewal(&self, rule: RefreshRule) -> Result<Renewal<'_>, StudioError> {
        let guard = self.auth.lock().await;
        let token = self.load()?;
        if let RefreshRule::UnexpiredAfter(age) = rule {
            let obtained = time::OffsetDateTime::parse(
                &token.obtained_at,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| StudioError::SignInRejected)?;
            if token.is_expired_at(&self.clock.now_rfc3339()) || self.clock.now() - obtained < age {
                return Err(StudioError::SignInRejected);
            }
        }
        Ok(Renewal {
            generation: self.generation(),
            token,
            _guard: guard,
        })
    }
    pub fn new(account: String, store: Arc<dyn TokenStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            account,
            store,
            clock,
            auth: Default::default(),
            writes: Default::default(),
        }
    }
    pub fn account(&self) -> String {
        self.account.clone()
    }
    pub fn invalidate(&self) {
        self.writes.invalidate();
    }
    pub fn generation(&self) -> u64 {
        self.writes.generation()
    }
    pub fn load(&self) -> Result<Token, StudioError> {
        self.store
            .load(&self.account)?
            .ok_or(StudioError::SignInRejected)
    }
    pub fn usable(&self) -> Result<Token, PostFailure> {
        crate::credentials::usable_token(
            self.store.as_ref(),
            &self.account,
            &self.clock.now_rfc3339(),
        )
    }
    pub fn connection(&self, pending: bool) -> Result<Connection, StudioError> {
        Ok(Connection::from_token(
            self.store.load(&self.account)?,
            &self.clock.now_rfc3339(),
            pending,
        ))
    }
    pub fn clear(&self) -> Result<Connection, StudioError> {
        self.writes.delete(self.store.as_ref(), &self.account)?;
        self.connection(false)
    }
    pub fn commit(
        &self,
        generation: u64,
        access_token: Secret,
        expires_at: String,
        member_id: String,
        scope: String,
    ) -> Result<(), StudioError> {
        let token = Token {
            access_token,
            expires_at: Some(expires_at),
            member_id,
            scope,
            obtained_at: self.clock.now_rfc3339(),
        };
        self.writes
            .save(generation, self.store.as_ref(), &self.account, &token)
    }
}
