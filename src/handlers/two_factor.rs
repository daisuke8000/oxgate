use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::repositories::User2faSecretRepository;
use crate::services::TotpService;
use crate::services::auth::AuthService;
use crate::state::AppState;

// === 2FA Setup ===

#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    pub user_id: Uuid,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub secret: String,
    pub qr_code: String,
}

/// POST /api/2fa/setup
///
/// 2FA設定を開始（シークレット生成、QRコード返却）
///
/// # Security
/// - パスワード確認必須
/// - シークレット平文はログ出力禁止
pub async fn setup_2fa(
    State(state): State<AppState>,
    Json(request): Json<SetupRequest>,
) -> Result<Json<SetupResponse>, AppError> {
    // バリデーション
    validate_password(&request.password)?;

    // パスワード確認
    let user = verify_user_password(&state, request.user_id, &request.password).await?;

    // 既に2FA設定済みかチェック
    let user_2fa_repo = User2faSecretRepository::new(state.db_pool.clone());
    if let Some(existing) = user_2fa_repo.find_by_user_id(user.id).await? {
        if existing.enabled {
            return Err(AppError::TotpAlreadyEnabled);
        }
        // enabled=false の場合は再設定を許可するため削除
        user_2fa_repo.delete(user.id).await?;
    }

    // シークレット生成
    let secret = TotpService::generate_secret();

    // 暗号化してDB保存
    let encrypted = state.totp_service.encrypt_secret(&secret)?;
    user_2fa_repo.create(user.id, &encrypted).await?;

    // QRコード生成
    let qr_code = state.totp_service.generate_qr_code(&user.email, &secret)?;

    tracing::info!(user_id = %user.id, "2FA設定開始");

    Ok(Json(SetupResponse {
        secret,
        qr_code: format!("data:image/png;base64,{}", qr_code),
    }))
}

// === 2FA Verify ===

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub user_id: Uuid,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub enabled: bool,
}

/// POST /api/2fa/verify
///
/// 2FA設定確認（初回コード検証で有効化）
///
/// # Security
/// - コードはログ出力禁止
pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    // バリデーション
    validate_totp_code(&request.code)?;

    // 2FAシークレット取得
    let user_2fa_repo = User2faSecretRepository::new(state.db_pool.clone());
    let user_2fa = user_2fa_repo
        .find_by_user_id(request.user_id)
        .await?
        .ok_or(AppError::TotpNotEnabled)?;

    if user_2fa.enabled {
        return Err(AppError::TotpAlreadyEnabled);
    }

    // シークレット復号
    let secret = state
        .totp_service
        .decrypt_secret(&user_2fa.secret_encrypted)?;

    // コード検証
    if !state.totp_service.verify_code(&secret, &request.code)? {
        return Err(AppError::TotpInvalid);
    }

    // 2FAを有効化
    user_2fa_repo.enable(request.user_id).await?;

    tracing::info!(user_id = %request.user_id, "2FA有効化完了");

    Ok(Json(VerifyResponse { enabled: true }))
}

// === 2FA Disable ===

#[derive(Debug, Deserialize)]
pub struct DisableRequest {
    pub user_id: Uuid,
    pub password: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct DisableResponse {
    pub disabled: bool,
}

/// POST /api/2fa/disable
///
/// 2FA無効化
///
/// # Security
/// - パスワード確認必須
/// - TOTPコード確認必須
pub async fn disable_2fa(
    State(state): State<AppState>,
    Json(request): Json<DisableRequest>,
) -> Result<Json<DisableResponse>, AppError> {
    // バリデーション
    validate_password(&request.password)?;
    validate_totp_code(&request.code)?;

    // パスワード確認
    let user = verify_user_password(&state, request.user_id, &request.password).await?;

    // 2FAシークレット取得
    let user_2fa_repo = User2faSecretRepository::new(state.db_pool.clone());
    let user_2fa = user_2fa_repo
        .find_by_user_id(user.id)
        .await?
        .ok_or(AppError::TotpNotEnabled)?;

    if !user_2fa.enabled {
        return Err(AppError::TotpNotEnabled);
    }

    // シークレット復号
    let secret = state
        .totp_service
        .decrypt_secret(&user_2fa.secret_encrypted)?;

    // コード検証
    if !state.totp_service.verify_code(&secret, &request.code)? {
        return Err(AppError::TotpInvalid);
    }

    // 2FAを削除
    user_2fa_repo.delete(user.id).await?;

    tracing::info!(user_id = %user.id, "2FA無効化完了");

    Ok(Json(DisableResponse { disabled: true }))
}

// === Helper Functions ===

/// パスワードバリデーション
fn validate_password(password: &str) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError::Validation("パスワードは必須です".to_string()));
    }
    if password.len() < 8 {
        return Err(AppError::Validation(
            "パスワードは8文字以上で入力してください".to_string(),
        ));
    }
    Ok(())
}

/// TOTPコードバリデーション
fn validate_totp_code(code: &str) -> Result<(), AppError> {
    if code.is_empty() {
        return Err(AppError::Validation("認証コードは必須です".to_string()));
    }
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            "認証コードは6桁の数字で入力してください".to_string(),
        ));
    }
    Ok(())
}

/// ユーザーのパスワードを検証し、ユーザー情報を返す
async fn verify_user_password(
    state: &AppState,
    user_id: Uuid,
    password: &str,
) -> Result<crate::models::User, AppError> {
    // ユーザー取得
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::Authentication("user not found".to_string()))?;

    // パスワード検証
    let auth_service = AuthService::new(state.user_repo.clone());
    auth_service.authenticate(&user.email, password).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_password() {
        let result = validate_password("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_short_password() {
        let result = validate_password("short");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_password() {
        let result = validate_password("password123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_code() {
        let result = validate_totp_code("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_short_code() {
        let result = validate_totp_code("12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_non_digit_code() {
        let result = validate_totp_code("12345a");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_code() {
        let result = validate_totp_code("123456");
        assert!(result.is_ok());
    }
}
