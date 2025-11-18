//! Authentication storage layer

use super::models::{Provider, Session, SessionId, SocialAccount, User, UserId};
use crate::storage::StorageResult;
use sled::Db;
use tracing::trace;

/// Trait for storing and retrieving authentication data
#[async_trait::async_trait]
pub trait AuthStorage: Send + Sync {
    /// Store a user
    async fn store_user(&self, user: &User) -> StorageResult<()>;

    /// Get a user by ID
    async fn get_user(&self, id: UserId) -> StorageResult<Option<User>>;

    /// Get a user by email
    async fn get_user_by_email(&self, email: &str) -> StorageResult<Option<User>>;

    /// Store a social account
    async fn store_social_account(&self, account: &SocialAccount) -> StorageResult<()>;

    /// Get a social account by provider and provider user ID
    async fn get_social_account(
        &self,
        provider: Provider,
        provider_user_id: &str,
    ) -> StorageResult<Option<SocialAccount>>;

    /// Get social accounts for a user
    async fn get_user_social_accounts(&self, user_id: UserId) -> StorageResult<Vec<SocialAccount>>;

    /// Store a session
    async fn store_session(&self, session: &Session) -> StorageResult<()>;

    /// Get a session by ID
    async fn get_session(&self, id: &SessionId) -> StorageResult<Option<Session>>;

    /// Delete a session
    async fn delete_session(&self, id: &SessionId) -> StorageResult<()>;

    /// Delete expired sessions
    async fn delete_expired_sessions(&self) -> StorageResult<usize>;
}

/// Sled-based authentication storage implementation
pub struct SledAuthStorage {
    db: Db,
}

impl SledAuthStorage {
    /// Create a new Sled auth storage instance
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Get the users tree
    fn users_tree(&self) -> StorageResult<sled::Tree> {
        Ok(self.db.open_tree("users")?)
    }

    /// Get the user email index tree
    fn user_email_index_tree(&self) -> StorageResult<sled::Tree> {
        Ok(self.db.open_tree("user_email_index")?)
    }

    /// Get the social accounts tree
    fn social_accounts_tree(&self) -> StorageResult<sled::Tree> {
        Ok(self.db.open_tree("social_accounts")?)
    }

    /// Get the user social accounts index tree
    fn user_social_accounts_tree(&self) -> StorageResult<sled::Tree> {
        Ok(self.db.open_tree("user_social_accounts")?)
    }

    /// Get the sessions tree
    fn sessions_tree(&self) -> StorageResult<sled::Tree> {
        Ok(self.db.open_tree("sessions")?)
    }
}

#[async_trait::async_trait]
impl AuthStorage for SledAuthStorage {
    async fn store_user(&self, user: &User) -> StorageResult<()> {
        let users_tree = self.users_tree()?;
        let email_index_tree = self.user_email_index_tree()?;

        let user_key = user.id.to_string();
        let user_value = serde_json::to_vec(user)?;

        // Store user
        users_tree.insert(user_key.as_bytes(), user_value)?;

        // Index by email
        email_index_tree.insert(user.email.as_bytes(), user_key.as_bytes())?;

        Ok(())
    }

    async fn get_user(&self, id: UserId) -> StorageResult<Option<User>> {
        let tree = self.users_tree()?;
        let key = id.to_string();

        if let Some(value) = tree.get(key.as_bytes())? {
            let user = serde_json::from_slice(&value)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn get_user_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        let email_index_tree = self.user_email_index_tree()?;
        let users_tree = self.users_tree()?;

        if let Some(user_id_bytes) = email_index_tree.get(email.as_bytes())? {
            if let Some(user_value) = users_tree.get(user_id_bytes.as_ref())? {
                let user = serde_json::from_slice(&user_value)?;
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn store_social_account(&self, account: &SocialAccount) -> StorageResult<()> {
        let social_tree = self.social_accounts_tree()?;
        let user_social_tree = self.user_social_accounts_tree()?;

        let account_key = account.key();
        let account_value = serde_json::to_vec(account)?;

        // Store social account
        social_tree.insert(account_key.as_bytes(), account_value)?;

        // Index by user ID
        let user_index_key = format!("{}:{}", account.user_id, account_key);
        user_social_tree.insert(user_index_key.as_bytes(), account_key.as_bytes())?;

        Ok(())
    }

    async fn get_social_account(
        &self,
        provider: Provider,
        provider_user_id: &str,
    ) -> StorageResult<Option<SocialAccount>> {
        let tree = self.social_accounts_tree()?;
        let key = format!("{}:{}", provider.as_str(), provider_user_id);

        if let Some(value) = tree.get(key.as_bytes())? {
            let account = serde_json::from_slice(&value)?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    async fn get_user_social_accounts(&self, user_id: UserId) -> StorageResult<Vec<SocialAccount>> {
        let user_social_tree = self.user_social_accounts_tree()?;
        let social_tree = self.social_accounts_tree()?;

        let prefix = format!("{}:", user_id);
        let mut accounts = Vec::new();

        for result in user_social_tree.scan_prefix(prefix.as_bytes()) {
            let (_key, account_key_bytes) = result?;
            if let Some(account_value) = social_tree.get(account_key_bytes.as_ref())? {
                if let Ok(account) = serde_json::from_slice(&account_value) {
                    accounts.push(account);
                }
            }
        }

        Ok(accounts)
    }

    async fn store_session(&self, session: &Session) -> StorageResult<()> {
        let tree = self.sessions_tree()?;
        let key = &session.id;
        let value = serde_json::to_vec(session)?;
        tree.insert(key.as_bytes(), value)?;
        Ok(())
    }

    async fn get_session(&self, id: &SessionId) -> StorageResult<Option<Session>> {
        let tree = self.sessions_tree()?;
        trace!("Retrieving session: {}", id);

        if let Some(value) = tree.get(id.as_bytes())? {
            let session: Session = serde_json::from_slice(&value)?;
            // Check if session is expired
            if session.is_expired() {
                trace!("Session {} is expired, deleting", id);
                tree.remove(id.as_bytes())?;
                return Ok(None);
            }
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn delete_session(&self, id: &SessionId) -> StorageResult<()> {
        let tree = self.sessions_tree()?;
        tree.remove(id.as_bytes())?;
        Ok(())
    }

    async fn delete_expired_sessions(&self) -> StorageResult<usize> {
        let tree = self.sessions_tree()?;
        let mut deleted = 0;

        for result in tree.iter() {
            let (key, value) = result?;
            if let Ok(session) = serde_json::from_slice::<Session>(&value) {
                if session.is_expired() {
                    tree.remove(key)?;
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }
}
