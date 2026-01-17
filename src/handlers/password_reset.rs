use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::services::PasswordResetService;
use crate::state::AppState;

// === リセットリクエスト ===

#[derive(Debug, Deserialize)]
pub struct ResetRequestRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ResetRequestResponse {
    pub message: String,
}

/// POST /api/password/reset-request
///
/// # Security
/// 常に200を返す（ユーザー存在有無を漏洩しない）
pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(request): Json<ResetRequestRequest>,
) -> Result<Json<ResetRequestResponse>, AppError> {
    // バリデーション
    validate_email(&request.email)?;

    // リセット処理（ユーザー不在でもエラーにしない）
    let password_reset_service = PasswordResetService::new(
        state.user_repo.clone(),
        state.token_repo.clone(),
        state.email_service.clone(),
        state.config.clone(),
    );
    password_reset_service.request_reset(&request.email).await?;

    Ok(Json(ResetRequestResponse {
        message: "パスワードリセット手順をメールで送信しました".to_string(),
    }))
}

// === パスワードリセット実行 ===

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

/// POST /api/password/reset
///
/// # Security
/// - token, new_password はログに出力しない
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, AppError> {
    // バリデーション
    validate_reset_password_request(&request)?;

    // リセット処理
    let password_reset_service = PasswordResetService::new(
        state.user_repo.clone(),
        state.token_repo.clone(),
        state.email_service.clone(),
        state.config.clone(),
    );
    password_reset_service
        .reset_password(&request.token, &request.new_password)
        .await?;

    tracing::info!("パスワードリセット完了");

    Ok(Json(ResetPasswordResponse {
        message: "パスワードが更新されました".to_string(),
    }))
}

/// メールアドレスのバリデーション
fn validate_email(email: &str) -> Result<(), AppError> {
    if email.trim().is_empty() || !email.contains('@') {
        return Err(AppError::Validation(
            "有効なメールアドレスを入力してください".to_string(),
        ));
    }
    Ok(())
}

/// リセットパスワードリクエストのバリデーション
fn validate_reset_password_request(request: &ResetPasswordRequest) -> Result<(), AppError> {
    if request.token.trim().is_empty() {
        return Err(AppError::Validation("トークンは必須です".to_string()));
    }
    if request.new_password.len() < 8 {
        return Err(AppError::Validation(
            "パスワードは8文字以上で入力してください".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_email() {
        let result = validate_email("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_email() {
        let result = validate_email("invalid-email");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_email() {
        let result = validate_email("test@example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_token() {
        let request = ResetPasswordRequest {
            token: "".to_string(),
            new_password: "password123".to_string(),
        };
        let result = validate_reset_password_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_short_password() {
        let request = ResetPasswordRequest {
            token: "valid-token".to_string(),
            new_password: "short".to_string(),
        };
        let result = validate_reset_password_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_reset_request() {
        let request = ResetPasswordRequest {
            token: "valid-token".to_string(),
            new_password: "password123".to_string(),
        };
        let result = validate_reset_password_request(&request);
        assert!(result.is_ok());
    }
}
