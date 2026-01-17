use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

use crate::config::Config;
use crate::error::AppError;
use crate::repositories::{PasswordResetTokenRepository, UserRepository};
use crate::services::{EmailService, auth::hash_password};

/// パスワードリセットサービス
#[derive(Clone)]
pub struct PasswordResetService {
    user_repo: UserRepository,
    token_repo: PasswordResetTokenRepository,
    email_service: EmailService,
    config: Arc<Config>,
}

impl PasswordResetService {
    /// 新しい PasswordResetService を作成
    pub fn new(
        user_repo: UserRepository,
        token_repo: PasswordResetTokenRepository,
        email_service: EmailService,
        config: Arc<Config>,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            email_service,
            config,
        }
    }

    /// パスワードリセットをリクエスト
    ///
    /// # Security
    /// - ユーザーが存在しない場合も常に成功を返す（情報漏洩防止）
    /// - トークン（平文）はログに出力しない
    pub async fn request_reset(&self, email: &str) -> Result<(), AppError> {
        tracing::info!(email = %email, "パスワードリセットリクエスト");

        // ユーザー検索
        let user = self.user_repo.find_by_email(email).await?;

        // ユーザーが存在しない場合も成功を返す（情報漏洩防止）
        let user = match user {
            Some(u) => u,
            None => {
                tracing::info!(email = %email, "パスワードリセット: ユーザー不在（成功レスポンス返却）");
                return Ok(());
            }
        };

        // 32バイトランダムトークン生成
        let token = self.generate_token()?;

        // SHA256ハッシュ化
        let token_hash = self.hash_token(&token);

        // 有効期限を設定
        let expires_at = OffsetDateTime::now_utc()
            + Duration::seconds(self.config.password_reset_token_ttl_secs);

        // トークンをDBに保存
        self.token_repo
            .create(user.id, &token_hash, expires_at)
            .await?;

        // リセットURLを構築
        let reset_url = self.build_reset_url(&token);

        // メール送信
        self.email_service
            .send_password_reset_email(email, &reset_url)
            .await?;

        tracing::info!(email = %email, "パスワードリセットメール送信完了");

        Ok(())
    }

    /// パスワードをリセット
    ///
    /// # Security
    /// - トークン・新パスワードはログに出力しない
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AppError> {
        // トークンをSHA256ハッシュ化
        let token_hash = self.hash_token(token);

        // DBからトークン検索
        let reset_token = self
            .token_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AppError::TokenNotFound)?;

        // 使用済みチェック
        if reset_token.used_at.is_some() {
            tracing::warn!(token_id = %reset_token.id, "使用済みトークン");
            return Err(AppError::TokenExpired);
        }

        // 有効期限チェック
        if reset_token.expires_at < OffsetDateTime::now_utc() {
            tracing::warn!(token_id = %reset_token.id, "期限切れトークン");
            return Err(AppError::TokenExpired);
        }

        // パスワードをargon2ハッシュ化
        let password_hash = hash_password(new_password)?;

        // パスワードを更新
        self.user_repo
            .update_password(reset_token.user_id, &password_hash)
            .await?;

        // トークンを使用済みにマーク
        self.token_repo.mark_as_used(reset_token.id).await?;

        tracing::info!(user_id = %reset_token.user_id, "パスワードリセット完了");

        Ok(())
    }

    /// 32バイトのランダムトークンを生成
    fn generate_token(&self) -> Result<String, AppError> {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }

    /// トークンをSHA256でハッシュ化
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// リセットURLを構築
    fn build_reset_url(&self, token: &str) -> String {
        match &self.config.password_reset_url_base {
            Some(base) => format!("{}?token={}", base, token),
            None => format!("http://localhost:3000/password-reset?token={}", token),
        }
    }
}
