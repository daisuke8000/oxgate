use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::repositories::{User2faSecretRepository, UserRepository};
use crate::services::auth::AuthService;
use crate::state::AppState;

/// ログインリクエスト
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Hydra から受け取ったログインチャレンジ
    pub login_challenge: String,
    /// ユーザーのメールアドレス
    pub email: String,
    /// ユーザーのパスワード
    pub password: String,
    /// 2FA認証コード（2FA有効ユーザーのみ必須）
    pub code: Option<String>,
}

/// ログインレスポンス
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// リダイレクト先URL（Hydra から返却される、2FA不要時のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_to: Option<String>,
    /// 2FAが必要かどうか
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_2fa: Option<bool>,
    /// ユーザーID（2FA必要時に返却）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<uuid::Uuid>,
}

/// ログインハンドラー
///
/// POST /api/login
///
/// 処理フロー:
/// 1. リクエストバリデーション
/// 2. Hydra でチャレンジ検証
/// 3. ユーザー認証（DB照合）
/// 4. 2FA有効チェック（有効なら requires_2fa: true を返却）
/// 5. 2FAコード検証（コードがある場合）
/// 6. Hydra でログイン承認
/// 7. リダイレクトURLを返却
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // 1. リクエストバリデーション
    validate_login_request(&request)?;

    // 2. Hydra でチャレンジ検証
    let login_info = state
        .hydra_client
        .get_login_request(&request.login_challenge)
        .await?;

    // skip=true の場合は以前の認証を再利用
    if login_info.skip {
        let redirect_to = state
            .hydra_client
            .accept_login(&request.login_challenge, &login_info.subject, true, 3600)
            .await?;

        return Ok(Json(LoginResponse {
            redirect_to: Some(redirect_to),
            requires_2fa: None,
            user_id: None,
        }));
    }

    // 3. ユーザー認証（DB照合）
    let user_repo = UserRepository::new(state.db_pool.clone());
    let auth_service = AuthService::new(user_repo);

    let user = auth_service
        .authenticate(&request.email, &request.password)
        .await?;

    // 4. 2FA有効チェック
    let user_2fa_repo = User2faSecretRepository::new(state.db_pool.clone());
    let user_2fa = user_2fa_repo.find_by_user_id(user.id).await?;

    if let Some(ref tfa) = user_2fa
        && tfa.enabled
    {
        // 2FAが有効なユーザー
        match &request.code {
            Some(code) => {
                // 5. 2FAコード検証
                validate_totp_code(code)?;
                let secret = state.totp_service.decrypt_secret(&tfa.secret_encrypted)?;
                if !state.totp_service.verify_code(&secret, code)? {
                    return Err(AppError::TotpInvalid);
                }
                // コード検証成功、ログイン続行
            }
            None => {
                // コードなし、2FA要求を返す
                return Ok(Json(LoginResponse {
                    redirect_to: None,
                    requires_2fa: Some(true),
                    user_id: Some(user.id),
                }));
            }
        }
    }

    // 6. Hydra でログイン承認
    let redirect_to = state
        .hydra_client
        .accept_login(&request.login_challenge, &user.id.to_string(), true, 3600)
        .await?;

    // 7. リダイレクトURLを返却
    Ok(Json(LoginResponse {
        redirect_to: Some(redirect_to),
        requires_2fa: None,
        user_id: None,
    }))
}

/// TOTPコードバリデーション
fn validate_totp_code(code: &str) -> Result<(), AppError> {
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            "認証コードは6桁の数字で入力してください".to_string(),
        ));
    }
    Ok(())
}

/// ログインリクエストのバリデーション
fn validate_login_request(request: &LoginRequest) -> Result<(), AppError> {
    // login_challenge: 必須、空文字不可
    if request.login_challenge.trim().is_empty() {
        return Err(AppError::Validation(
            "login_challenge は必須です".to_string(),
        ));
    }

    // email: 必須、メール形式
    if request.email.trim().is_empty() {
        return Err(AppError::Validation("メールアドレスは必須です".to_string()));
    }

    // 簡易的なメール形式チェック（@ が含まれているか）
    if !request.email.contains('@') {
        return Err(AppError::Validation(
            "有効なメールアドレスを入力してください".to_string(),
        ));
    }

    // password: 必須、8文字以上
    if request.password.is_empty() {
        return Err(AppError::Validation("パスワードは必須です".to_string()));
    }

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
    fn test_validate_empty_login_challenge() {
        let request = LoginRequest {
            login_challenge: "".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            code: None,
        };

        let result = validate_login_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_email() {
        let request = LoginRequest {
            login_challenge: "challenge123".to_string(),
            email: "".to_string(),
            password: "password123".to_string(),
            code: None,
        };

        let result = validate_login_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_email() {
        let request = LoginRequest {
            login_challenge: "challenge123".to_string(),
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            code: None,
        };

        let result = validate_login_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_short_password() {
        let request = LoginRequest {
            login_challenge: "challenge123".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
            code: None,
        };

        let result = validate_login_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_request() {
        let request = LoginRequest {
            login_challenge: "challenge123".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            code: None,
        };

        let result = validate_login_request(&request);
        assert!(result.is_ok());
    }
}
