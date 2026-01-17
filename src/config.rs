use secrecy::SecretBox;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: SecretBox<String>,
    pub hydra_admin_url: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,

    // SMTP設定（オプション - email機能有効時のみ使用）
    #[serde(default)]
    pub smtp_host: Option<String>,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    pub smtp_username: Option<SecretBox<String>>,
    pub smtp_password: Option<SecretBox<String>>,
    #[serde(default)]
    pub smtp_from_address: Option<String>,

    // パスワードリセット設定
    #[serde(default)]
    pub password_reset_url_base: Option<String>,
    #[serde(default = "default_password_reset_token_ttl_secs")]
    pub password_reset_token_ttl_secs: i64,

    // 2FA (TOTP) 設定
    /// TOTP発行者名（認証アプリに表示される）
    pub totp_issuer: String,
    /// AES-256暗号化キー（Base64エンコード、32バイト）
    pub encryption_key: SecretBox<String>,

    // OAuth2 ソーシャルログイン設定
    /// OAuthステート暗号化用シークレット（必須、32バイト推奨）
    pub oauth_state_secret: SecretBox<String>,

    // Google OAuth設定（オプション）
    #[serde(default)]
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<SecretBox<String>>,
    #[serde(default)]
    pub google_redirect_uri: Option<String>,

    // GitHub OAuth設定（オプション）
    #[serde(default)]
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<SecretBox<String>>,
    #[serde(default)]
    pub github_redirect_uri: Option<String>,
}

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_SMTP_PORT: u16 = 587;
const DEFAULT_PASSWORD_RESET_TOKEN_TTL_SECS: i64 = 3600;

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_smtp_port() -> u16 {
    DEFAULT_SMTP_PORT
}

fn default_password_reset_token_ttl_secs() -> i64 {
    DEFAULT_PASSWORD_RESET_TOKEN_TTL_SECS
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        envy::from_env()
    }
}
