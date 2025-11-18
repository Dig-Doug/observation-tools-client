//! Authentication module

pub mod handlers;
pub mod middleware;
pub mod models;
pub mod oauth;
pub mod storage;

pub use handlers::{AuthState, login_page, logout, oauth_callback, oauth_login};
pub use middleware::{extract_user, require_auth, AuthUser, SESSION_COOKIE_NAME};
pub use models::{Provider, Session, SessionId, SocialAccount, User, UserId};
pub use oauth::{OAuthConfig, OAuthManager, OAuthProvider, OAuthUserInfo};
pub use storage::{AuthStorage, SledAuthStorage};
