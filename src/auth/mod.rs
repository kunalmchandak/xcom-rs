/// OAuth2 login flow implementation
pub mod login;
/// Authentication models and data structures
pub mod models;
/// OAuth1.0a authentication flow
pub mod oauth1a;
/// OAuth2 authentication flow
pub mod oauth2;
/// Authentication storage and persistence
pub mod storage;

pub use login::{local_server_login, manual_login, parse_redirect_url};
pub use models::{AuthCredentials, AuthStatus, OAuth1aCredentials, OAuth2Credentials};
pub use oauth1a::{AccessTokenResponse, OAuth1aClient, RequestTokenResponse};
pub use oauth2::{generate_state, AuthorizeParams, OAuth2Client, Pkce, TokenResponse};
pub use storage::AuthStore;
