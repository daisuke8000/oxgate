use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

/// 同意リクエスト
#[derive(Debug, Deserialize)]
pub struct ConsentRequest {
    /// Hydra から受け取った同意チャレンジ
    pub consent_challenge: String,
    /// ユーザーが許可するスコープ
    pub grant_scope: Vec<String>,
}

/// 同意レスポンス
#[derive(Debug, Serialize)]
pub struct ConsentResponse {
    /// リダイレクト先URL（Hydra から返却される）
    pub redirect_to: String,
}

/// 同意ハンドラー
///
/// POST /api/consent
///
/// 処理フロー:
/// 1. リクエストバリデーション
/// 2. Hydra でチャレンジ検証
/// 3. skip=true なら以前の同意を再利用
/// 4. grant_scope のバリデーション（requested_scope のサブセットか）
/// 5. Hydra で同意承認
/// 6. リダイレクトURLを返却
pub async fn consent(
    State(state): State<AppState>,
    Json(request): Json<ConsentRequest>,
) -> Result<Json<ConsentResponse>, AppError> {
    // 1. リクエストバリデーション
    validate_consent_request(&request)?;

    // 2. Hydra でチャレンジ検証
    let consent_info = state
        .hydra_client
        .get_consent_request(&request.consent_challenge)
        .await?;

    // 3. skip=true の場合は以前の同意を再利用
    if consent_info.skip {
        let redirect_to = state
            .hydra_client
            .accept_consent(
                &request.consent_challenge,
                consent_info.requested_scope.clone(),
                consent_info.requested_access_token_audience.clone(),
                true,
                3600,
                None,
            )
            .await?;

        tracing::info!(
            subject = %consent_info.subject,
            client_id = %consent_info.client.client_id,
            "同意スキップ（以前の同意を再利用）"
        );

        return Ok(Json(ConsentResponse { redirect_to }));
    }

    // 4. grant_scope のバリデーション（requested_scope のサブセットか）
    validate_grant_scope(&request.grant_scope, &consent_info.requested_scope)?;

    // 5. Hydra で同意承認
    let redirect_to = state
        .hydra_client
        .accept_consent(
            &request.consent_challenge,
            request.grant_scope.clone(),
            consent_info.requested_access_token_audience.clone(),
            true,
            3600,
            None,
        )
        .await?;

    tracing::info!(
        subject = %consent_info.subject,
        client_id = %consent_info.client.client_id,
        granted_scopes = ?request.grant_scope,
        "同意承認完了"
    );

    // 6. リダイレクトURLを返却
    Ok(Json(ConsentResponse { redirect_to }))
}

/// 同意リクエストのバリデーション
fn validate_consent_request(request: &ConsentRequest) -> Result<(), AppError> {
    // consent_challenge: 必須、空文字不可
    if request.consent_challenge.trim().is_empty() {
        return Err(AppError::Validation(
            "consent_challenge は必須です".to_string(),
        ));
    }

    Ok(())
}

/// grant_scope が requested_scope のサブセットかを検証
fn validate_grant_scope(
    grant_scope: &[String],
    requested_scope: &[String],
) -> Result<(), AppError> {
    for scope in grant_scope {
        if !requested_scope.contains(scope) {
            return Err(AppError::Validation(format!(
                "スコープ '{}' はリクエストされていません",
                scope
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_consent_challenge() {
        let request = ConsentRequest {
            consent_challenge: "".to_string(),
            grant_scope: vec!["openid".to_string()],
        };

        let result = validate_consent_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_whitespace_consent_challenge() {
        let request = ConsentRequest {
            consent_challenge: "   ".to_string(),
            grant_scope: vec!["openid".to_string()],
        };

        let result = validate_consent_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_consent_request() {
        let request = ConsentRequest {
            consent_challenge: "challenge123".to_string(),
            grant_scope: vec!["openid".to_string()],
        };

        let result = validate_consent_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_grant_scope_valid_subset() {
        let grant_scope = vec!["openid".to_string(), "profile".to_string()];
        let requested_scope = vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ];

        let result = validate_grant_scope(&grant_scope, &requested_scope);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_grant_scope_empty_grant() {
        let grant_scope: Vec<String> = vec![];
        let requested_scope = vec!["openid".to_string(), "profile".to_string()];

        let result = validate_grant_scope(&grant_scope, &requested_scope);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_grant_scope_invalid_scope() {
        let grant_scope = vec!["openid".to_string(), "admin".to_string()];
        let requested_scope = vec!["openid".to_string(), "profile".to_string()];

        let result = validate_grant_scope(&grant_scope, &requested_scope);
        assert!(result.is_err());

        if let Err(AppError::Validation(msg)) = result {
            assert!(msg.contains("admin"));
        }
    }

    #[test]
    fn test_validate_grant_scope_exact_match() {
        let grant_scope = vec!["openid".to_string(), "profile".to_string()];
        let requested_scope = vec!["openid".to_string(), "profile".to_string()];

        let result = validate_grant_scope(&grant_scope, &requested_scope);
        assert!(result.is_ok());
    }
}
