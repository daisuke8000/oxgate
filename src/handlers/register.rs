use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;
use crate::repositories::UserRepository;
use crate::services::auth::hash_password;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String, // SecretBox不要（Deserialize後すぐハッシュ化）
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// ユーザー登録ハンドラー
///
/// # Security
/// - パスワードはログに出力しない
/// - パスワードは即座にハッシュ化
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    // バリデーション
    validate_register_request(&request)?;

    // パスワードハッシュ化
    let password_hash = hash_password(&request.password)?;

    // ユーザー作成
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo
        .create_user(&request.email, &password_hash)
        .await
        .map_err(|e| {
            // UNIQUE制約違反チェック
            if let sqlx::Error::Database(db_err) = &e
                && db_err.constraint() == Some("users_email_key")
            {
                return AppError::EmailAlreadyExists;
            }
            AppError::Database(e)
        })?;

    tracing::info!(email = %request.email, "ユーザー登録成功");

    Ok(Json(RegisterResponse {
        id: user.id,
        email: user.email,
        created_at: user.created_at,
    }))
}

/// 登録リクエストのバリデーション
fn validate_register_request(request: &RegisterRequest) -> Result<(), AppError> {
    // email: 必須、メール形式
    if request.email.trim().is_empty() {
        return Err(AppError::Validation("メールアドレスは必須です".to_string()));
    }
    if !request.email.contains('@') {
        return Err(AppError::Validation(
            "有効なメールアドレスを入力してください".to_string(),
        ));
    }
    // password: 8文字以上
    if request.password.len() < 8 {
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
        let request = RegisterRequest {
            email: "".to_string(),
            password: "password123".to_string(),
        };
        let result = validate_register_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_email() {
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        let result = validate_register_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_short_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        let result = validate_register_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_request() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        let result = validate_register_request(&request);
        assert!(result.is_ok());
    }
}
