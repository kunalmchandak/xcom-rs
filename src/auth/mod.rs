/// OAuth2 login flow implementation
pub mod login;
/// Authentication models and data structures
pub mod models;
/// OAuth2 authentication flow
pub mod oauth2;
/// Authentication storage and persistence
pub mod storage;

pub use login::{local_server_login, manual_login, parse_redirect_url};
pub use models::{AuthStatus, OAuth2Credentials};
pub use oauth2::{generate_state, AuthorizeParams, OAuth2Client, Pkce, TokenResponse};
pub use storage::AuthStore;
