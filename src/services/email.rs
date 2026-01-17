use std::sync::Arc;

use crate::config::Config;
use crate::error::AppError;

/// メール送信サービス（開発環境: スタブ実装）
#[derive(Clone)]
pub struct EmailService {
    config: Arc<Config>,
}

impl EmailService {
    /// 新しい EmailService を作成
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// パスワードリセットメールを送信（開発環境: ログ出力のみ）
    ///
    /// 本番環境では lettre クレートを使用してメール送信を実装予定
    pub async fn send_password_reset_email(
        &self,
        to: &str,
        reset_url: &str,
    ) -> Result<(), AppError> {
        // 開発モード: メール送信せずログ出力のみ
        tracing::info!(
            to = %to,
            "パスワードリセットメール送信（開発モード）"
        );
        tracing::info!("リセットURL: {}", reset_url);

        // 本番環境では lettre を使用してメール送信
        // SMTP設定が存在するか確認
        let _smtp_configured = self.config.smtp_host.is_some()
            && self.config.smtp_username.is_some()
            && self.config.smtp_password.is_some()
            && self.config.smtp_from_address.is_some();

        // TODO: 本番実装時は以下のような形式で lettre を使用
        // if smtp_configured {
        //     let mailer = SmtpTransport::relay(host)?.build();
        //     mailer.send(&email)?;
        // }

        Ok(())
    }
}
