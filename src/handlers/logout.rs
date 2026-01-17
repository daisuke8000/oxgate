use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

/// ログアウトリクエスト
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// Hydra から受け取ったログアウトチャレンジ
    pub logout_challenge: String,
}

/// ログアウトレスポンス
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    /// リダイレクト先URL（Hydra から返却される）
    pub redirect_to: String,
}

/// ログアウトハンドラー
///
/// POST /api/logout
///
/// 処理フロー:
/// 1. リクエストバリデーション
/// 2. Hydra でチャレンジ検証
/// 3. Hydra でログアウト承認
/// 4. リダイレクトURLを返却
pub async fn logout(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, AppError> {
    // 1. リクエストバリデーション
    validate_logout_request(&request)?;

    // 2. Hydra でチャレンジ検証
    let logout_info = state
        .hydra_client
        .get_logout_request(&request.logout_challenge)
        .await?;

    // 3. Hydra でログアウト承認
    let redirect_to = state
        .hydra_client
        .accept_logout(&request.logout_challenge)
        .await?;

    tracing::info!(
        subject = %logout_info.subject,
        "ログアウト完了"
    );

    // 4. リダイレクトURLを返却
    Ok(Json(LogoutResponse { redirect_to }))
}

/// ログアウトリクエストのバリデーション
fn validate_logout_request(request: &LogoutRequest) -> Result<(), AppError> {
    // logout_challenge: 必須、空文字不可
    if request.logout_challenge.trim().is_empty() {
        return Err(AppError::Validation(
            "logout_challenge は必須です".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_logout_challenge() {
        let request = LogoutRequest {
            logout_challenge: "".to_string(),
        };

        let result = validate_logout_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_whitespace_logout_challenge() {
        let request = LogoutRequest {
            logout_challenge: "   ".to_string(),
        };

        let result = validate_logout_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_logout_request() {
        let request = LogoutRequest {
            logout_challenge: "challenge123".to_string(),
        };

        let result = validate_logout_request(&request);
        assert!(result.is_ok());
    }
}
