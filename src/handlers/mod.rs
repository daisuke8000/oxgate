pub mod consent;
pub mod health;
pub mod login;
pub mod logout;
pub mod oauth;
pub mod password_reset;
pub mod register;
pub mod two_factor;

pub use consent::consent;
pub use health::health_check;
pub use login::login;
pub use logout::logout;
pub use oauth::{github_auth, github_callback, google_auth, google_callback};
pub use password_reset::{request_password_reset, reset_password};
pub use register::register;
pub use two_factor::{disable_2fa, setup_2fa, verify_2fa};
