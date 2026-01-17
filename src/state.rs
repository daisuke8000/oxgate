use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
use crate::error::AppError;
use crate::repositories::{
    PasswordResetTokenRepository, User2faSecretRepository, UserRepository,
    UserSocialAccountRepository,
};
use crate::services::hydra::HydraClient;
use crate::services::{EmailService, GitHubOAuthService, OAuthService, TotpService};
use secrecy::ExposeSecret;

/// アプリケーション共有状態
///
/// axum の State として全ハンドラーで共有される。
/// Clone は必須（axum が内部で clone するため）。
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL コネクションプール
    pub db_pool: PgPool,
    /// Hydra Admin API クライアント
    pub hydra_client: HydraClient,
    /// アプリケーション設定（Arc で共有）
    pub config: Arc<Config>,
    /// ユーザーリポジトリ
    pub user_repo: UserRepository,
    /// パスワードリセットトークンリポジトリ
    pub token_repo: PasswordResetTokenRepository,
    /// メールサービス
    pub email_service: EmailService,
    /// 2FAシークレットリポジトリ
    pub user_2fa_repo: User2faSecretRepository,
    /// TOTPサービス
    pub totp_service: TotpService,
    /// ソーシャルアカウントリポジトリ
    pub social_account_repo: UserSocialAccountRepository,
    /// Google OAuth サービス（設定されている場合のみ）
    pub google_oauth_service: Option<OAuthService>,
    /// GitHub OAuth サービス（設定されている場合のみ）
    pub github_oauth_service: Option<GitHubOAuthService>,
}

impl AppState {
    /// 新しい AppState を作成
    pub fn new(
        db_pool: PgPool,
        hydra_client: HydraClient,
        config: Config,
    ) -> Result<Self, AppError> {
        let config = Arc::new(config);
        let user_repo = UserRepository::new(db_pool.clone());
        let token_repo = PasswordResetTokenRepository::new(db_pool.clone());
        let email_service = EmailService::new(config.clone());
        let user_2fa_repo = User2faSecretRepository::new(db_pool.clone());
        let totp_service = TotpService::new(
            config.totp_issuer.clone(),
            config.encryption_key.expose_secret(),
        )?;

        let social_account_repo = UserSocialAccountRepository::new(db_pool.clone());

        // Google OAuth サービス（設定されている場合のみ初期化）
        let google_oauth_service = match (
            &config.google_client_id,
            &config.google_client_secret,
            &config.google_redirect_uri,
        ) {
            (Some(client_id), Some(client_secret), Some(redirect_uri)) => {
                tracing::info!("Google OAuth サービスを初期化");
                Some(OAuthService::new(
                    client_id.clone(),
                    client_secret.expose_secret().clone(),
                    redirect_uri.clone(),
                    config.oauth_state_secret.expose_secret(),
                )?)
            }
            _ => {
                tracing::info!("Google OAuth 未設定（スキップ）");
                None
            }
        };

        // GitHub OAuth サービス（設定されている場合のみ初期化）
        let github_oauth_service = match (
            &config.github_client_id,
            &config.github_client_secret,
            &config.github_redirect_uri,
        ) {
            (Some(client_id), Some(client_secret), Some(redirect_uri)) => {
                tracing::info!("GitHub OAuth サービスを初期化");
                Some(GitHubOAuthService::new(
                    client_id.clone(),
                    client_secret.expose_secret().clone(),
                    redirect_uri.clone(),
                    config.oauth_state_secret.expose_secret(),
                )?)
            }
            _ => {
                tracing::info!("GitHub OAuth 未設定（スキップ）");
                None
            }
        };

        Ok(Self {
            db_pool,
            hydra_client,
            config,
            user_repo,
            token_repo,
            email_service,
            user_2fa_repo,
            totp_service,
            social_account_repo,
            google_oauth_service,
            github_oauth_service,
        })
    }
}
