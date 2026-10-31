//! An in-memory token store for tests that do not exercise the Keychain itself.
#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use syncloom_core::{StoreError, Token, TokenStore};

#[derive(Default)]
pub struct MemoryStore {
    tokens: Mutex<HashMap<String, Token>>,
}

impl MemoryStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn with(account: &str, token: Token) -> Arc<Self> {
        let store = Self::default();
        store
            .tokens
            .lock()
            .unwrap()
            .insert(account.to_owned(), token);
        Arc::new(store)
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn get(&self, account: &str) -> Option<Token> {
        self.tokens.lock().unwrap().get(account).cloned()
    }
}

impl TokenStore for MemoryStore {
    fn load(&self, account: &str) -> Result<Option<Token>, StoreError> {
        Ok(self.get(account))
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    fn save(&self, account: &str, token: &Token) -> Result<(), StoreError> {
        self.tokens
            .lock()
            .unwrap()
            .insert(account.to_owned(), token.clone());
        Ok(())
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    fn delete(&self, account: &str) -> Result<(), StoreError> {
        self.tokens.lock().unwrap().remove(account);
        Ok(())
    }
}
