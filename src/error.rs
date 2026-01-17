use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("認証エラー: {0}")]
    Authentication(String),

    #[error("バリデーションエラー: {0}")]
    Validation(String),

    #[error("データベースエラー")]
    Database(#[from] sqlx::Error),

    #[error("Hydra API エラー")]
    Hydra(#[from] reqwest::Error),

    #[error("内部エラー")]
    Internal(#[from] anyhow::Error),

    #[error("このメールアドレスは既に使用されています")]
    EmailAlreadyExists,

    #[error("無効または期限切れのリンクです")]
    TokenExpired,

    #[error("トークンが見つかりません")]
    TokenNotFound,

    #[error("認証コードが無効です")]
    TotpInvalid,

    #[error("二要素認証は既に有効です")]
    TotpAlreadyEnabled,

    #[error("二要素認証が有効化されていません")]
    TotpNotEnabled,

    #[error("二要素認証の設定が必要です")]
    TotpSetupRequired,

    #[error("OAuth認証エラー: {0}")]
    OAuthError(String),

    #[error("無効なstateパラメータ")]
    OAuthStateInvalid,

    #[error("OAuthプロバイダーエラー")]
    OAuthProviderError,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Authentication(_) => (
                StatusCode::UNAUTHORIZED,
                "メールアドレスまたはパスワードが正しくありません".to_string(),
            ),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(e) => {
                tracing::error!(error = ?e, "データベースエラー");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部エラーが発生しました".to_string(),
                )
            }
            Self::Internal(e) => {
                tracing::error!(error = ?e, "内部エラー");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部エラーが発生しました".to_string(),
                )
            }
            Self::Hydra(e) => {
                tracing::error!(error = ?e, "Hydra通信エラー");
                (
                    StatusCode::BAD_GATEWAY,
                    "認証サーバーとの通信に失敗しました".to_string(),
                )
            }
            Self::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "このメールアドレスは既に使用されています".to_string(),
            ),
            Self::TokenExpired => (
                StatusCode::BAD_REQUEST,
                "無効または期限切れのリンクです".to_string(),
            ),
            Self::TokenNotFound => (
                StatusCode::BAD_REQUEST,
                "無効なリクエストです".to_string(), // 存在有無の漏洩防止
            ),
            Self::TotpInvalid => (
                StatusCode::UNAUTHORIZED,
                "認証コードが正しくありません".to_string(),
            ),
            Self::TotpAlreadyEnabled => {
                (StatusCode::CONFLICT, "二要素認証は既に有効です".to_string())
            }
            Self::TotpNotEnabled => (
                StatusCode::BAD_REQUEST,
                "二要素認証が有効化されていません".to_string(),
            ),
            Self::TotpSetupRequired => (
                StatusCode::FORBIDDEN,
                "二要素認証の設定が必要です".to_string(),
            ),
            Self::OAuthError(e) => {
                tracing::error!(error = %e, "OAuth認証エラー");
                (StatusCode::UNAUTHORIZED, "認証に失敗しました".to_string())
            }
            Self::OAuthStateInvalid => {
                tracing::warn!("無効なOAuth stateパラメータ（CSRF攻撃の可能性）");
                (StatusCode::BAD_REQUEST, "無効なリクエストです".to_string())
            }
            Self::OAuthProviderError => (
                StatusCode::BAD_GATEWAY,
                "外部認証サービスとの通信に失敗しました".to_string(),
            ),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
