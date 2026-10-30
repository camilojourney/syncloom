//! An in-memory token store for tests that do not exercise the Keychain itself.
#![allow(dead_code)]
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
    pub fn with(account: &str, token: Token) -> Arc<Self> {
        let store = Self::default();
        store
            .tokens
            .lock()
            .unwrap()
            .insert(account.to_owned(), token);
        Arc::new(store)
    }
    pub fn get(&self, account: &str) -> Option<Token> {
        self.tokens.lock().unwrap().get(account).cloned()
    }
}

impl TokenStore for MemoryStore {
    fn load(&self, account: &str) -> Result<Option<Token>, StoreError> {
        Ok(self.get(account))
    }
    fn save(&self, account: &str, token: &Token) -> Result<(), StoreError> {
        self.tokens
            .lock()
            .unwrap()
            .insert(account.to_owned(), token.clone());
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<(), StoreError> {
        self.tokens.lock().unwrap().remove(account);
        Ok(())
    }
}
