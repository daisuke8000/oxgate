//! OAuth ソーシャルログインハンドラー
//!
//! Google および GitHub OAuth を使用したソーシャルログイン処理を提供する。
//!
//! # Security
//! - state パラメータは AES-256-GCM で暗号化され、login_challenge を含む
//! - access_token はログに出力しない
//! - provider_id を使用してユーザーを一意に識別

use axum::{
    Json,
    extract::{Query, State},
    response::Redirect,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

/// OAuth 認証開始時のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct OAuthQuery {
    /// Hydra から受け取った login_challenge
    pub login_challenge: String,
}

/// OAuth コールバック時のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    /// OAuth プロバイダーから受け取った認可コード
    pub code: String,
    /// 暗号化された state（login_challenge を含む）
    pub state: String,
}

/// OAuth 認証 URL レスポンス
#[derive(Debug, Serialize)]
pub struct OAuthAuthResponse {
    /// OAuth 認可 URL（フロントエンドでリダイレクトに使用）
    pub auth_url: String,
}

/// OAuth コールバック成功レスポンス
#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    /// Hydra からのリダイレクト先 URL
    pub redirect_to: String,
}

// =============================================================================
// Google OAuth ハンドラー
// =============================================================================

/// Google OAuth 認証 URL を生成
///
/// フロントエンドはこの URL にユーザーをリダイレクトする。
///
/// # Arguments
/// * `state` - アプリケーション状態
/// * `query` - login_challenge を含むクエリパラメータ
///
/// # Returns
/// Google OAuth 認可 URL
pub async fn google_auth(
    State(state): State<AppState>,
    Query(query): Query<OAuthQuery>,
) -> Result<Json<OAuthAuthResponse>, AppError> {
    tracing::info!("Google OAuth 認証開始");

    let oauth_service = state.google_oauth_service.as_ref().ok_or_else(|| {
        tracing::warn!("Google OAuth が設定されていません");
        AppError::OAuthError("Google OAuth is not configured".to_string())
    })?;

    let auth_url = oauth_service.generate_auth_url(&query.login_challenge)?;

    tracing::debug!("Google OAuth 認可 URL 生成成功");
    Ok(Json(OAuthAuthResponse { auth_url }))
}

/// Google OAuth コールバック処理
///
/// # 処理フロー
/// 1. state をデコードして login_challenge を復元
/// 2. code でトークン交換
/// 3. access_token でユーザー情報取得
/// 4. provider_id で user_social_accounts 検索
///    - 見つかれば: 既存ユーザーでログイン
///    - 見つからなければ:
///      - email で users 検索
///      - 見つかれば: user_social_accounts を作成（紐付け）
///      - 見つからなければ: users 作成（create_social_user）+ user_social_accounts 作成
/// 5. Hydra login accept を呼び出し
/// 6. redirect_to にリダイレクト
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    tracing::info!("Google OAuth コールバック受信");

    let oauth_service = state.google_oauth_service.as_ref().ok_or_else(|| {
        tracing::warn!("Google OAuth が設定されていません");
        AppError::OAuthError("Google OAuth is not configured".to_string())
    })?;

    // 1. state をデコードして login_challenge を復元
    let login_challenge = oauth_service.decode_state(&query.state)?;
    tracing::debug!("state デコード成功");

    // 2. code でトークン交換
    let token_response = oauth_service.exchange_code(&query.code).await?;
    tracing::debug!("トークン交換成功");
    // Note: access_token はログに出力しない

    // 3. access_token でユーザー情報取得
    let user_info = oauth_service
        .get_user_info(&token_response.access_token)
        .await?;
    tracing::info!(provider = "google", "OAuth ユーザー情報取得成功");

    // 4-6. ユーザー処理と Hydra accept
    let redirect_to = process_oauth_callback(
        &state,
        "google",
        &user_info.id,
        &user_info.email,
        &login_challenge,
    )
    .await?;

    Ok(Redirect::to(&redirect_to))
}

// =============================================================================
// GitHub OAuth ハンドラー
// =============================================================================

/// GitHub OAuth 認証 URL を生成
///
/// フロントエンドはこの URL にユーザーをリダイレクトする。
///
/// # Arguments
/// * `state` - アプリケーション状態
/// * `query` - login_challenge を含むクエリパラメータ
///
/// # Returns
/// GitHub OAuth 認可 URL
pub async fn github_auth(
    State(state): State<AppState>,
    Query(query): Query<OAuthQuery>,
) -> Result<Json<OAuthAuthResponse>, AppError> {
    tracing::info!("GitHub OAuth 認証開始");

    let oauth_service = state.github_oauth_service.as_ref().ok_or_else(|| {
        tracing::warn!("GitHub OAuth が設定されていません");
        AppError::OAuthError("GitHub OAuth is not configured".to_string())
    })?;

    let auth_url = oauth_service.generate_auth_url(&query.login_challenge)?;

    tracing::debug!("GitHub OAuth 認可 URL 生成成功");
    Ok(Json(OAuthAuthResponse { auth_url }))
}

/// GitHub OAuth コールバック処理
///
/// # 処理フロー
/// Google OAuth と同様
pub async fn github_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    tracing::info!("GitHub OAuth コールバック受信");

    let oauth_service = state.github_oauth_service.as_ref().ok_or_else(|| {
        tracing::warn!("GitHub OAuth が設定されていません");
        AppError::OAuthError("GitHub OAuth is not configured".to_string())
    })?;

    // 1. state をデコードして login_challenge を復元
    let login_challenge = oauth_service.decode_state(&query.state)?;
    tracing::debug!("state デコード成功");

    // 2. code でトークン交換
    let token_response = oauth_service.exchange_code(&query.code).await?;
    tracing::debug!("トークン交換成功");
    // Note: access_token はログに出力しない

    // 3. access_token でユーザー情報取得
    let user_info = oauth_service
        .get_user_info(&token_response.access_token)
        .await?;
    tracing::info!(provider = "github", "OAuth ユーザー情報取得成功");

    // 4-6. ユーザー処理と Hydra accept
    let redirect_to = process_oauth_callback(
        &state,
        "github",
        &user_info.id,
        &user_info.email,
        &login_challenge,
    )
    .await?;

    Ok(Redirect::to(&redirect_to))
}

// =============================================================================
// 共通処理
// =============================================================================

/// OAuth コールバックの共通処理
///
/// # 処理フロー
/// 1. provider_id で user_social_accounts 検索
///    - 見つかれば: 既存ユーザーでログイン
///    - 見つからなければ:
///      - email で users 検索
///      - 見つかれば: user_social_accounts を作成（紐付け）
///      - 見つからなければ: users 作成（create_social_user）+ user_social_accounts 作成
/// 2. Hydra login accept を呼び出し
/// 3. redirect_to を返す
async fn process_oauth_callback(
    state: &AppState,
    provider: &str,
    provider_id: &str,
    email: &str,
    login_challenge: &str,
) -> Result<String, AppError> {
    // 4. provider_id で user_social_accounts 検索
    let existing_social_account = state
        .social_account_repo
        .find_by_provider_and_id(provider, provider_id)
        .await?;

    let user_id = match existing_social_account {
        Some(social_account) => {
            // 既存のソーシャルアカウントが見つかった
            tracing::info!(
                provider = %provider,
                user_id = %social_account.user_id,
                "既存ソーシャルアカウントでログイン"
            );
            social_account.user_id
        }
        None => {
            // ソーシャルアカウントが見つからない - ユーザーを検索または作成
            tracing::debug!(provider = %provider, "ソーシャルアカウント未登録 - ユーザー検索");

            let user = match state.user_repo.find_by_email(email).await? {
                Some(existing_user) => {
                    // メールアドレスで既存ユーザーが見つかった - 紐付け
                    tracing::info!(
                        provider = %provider,
                        user_id = %existing_user.id,
                        "既存ユーザーにソーシャルアカウントを紐付け"
                    );
                    existing_user
                }
                None => {
                    // 新規ユーザーを作成（パスワードなし）
                    tracing::info!(
                        provider = %provider,
                        "新規ソーシャルユーザーを作成"
                    );
                    state.user_repo.create_social_user(email).await?
                }
            };

            // ソーシャルアカウントを作成
            state
                .social_account_repo
                .create(user.id, provider, provider_id, Some(email))
                .await?;
            tracing::debug!(provider = %provider, "ソーシャルアカウント紐付け完了");

            user.id
        }
    };

    // 5. Hydra login accept を呼び出し
    let redirect_to = state
        .hydra_client
        .accept_login(
            login_challenge,
            &user_id.to_string(),
            true, // remember
            3600, // remember_for: 1時間
        )
        .await?;

    tracing::info!(
        provider = %provider,
        user_id = %user_id,
        "OAuth ログイン成功"
    );

    // 6. redirect_to を返す
    Ok(redirect_to)
}
