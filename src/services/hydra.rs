use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Hydra からのログインリクエスト情報
#[derive(Debug, Deserialize)]
pub struct HydraLoginRequest {
    pub challenge: String,
    pub skip: bool,
    pub subject: String,
    pub client: HydraClientInfo,
    pub request_url: String,
    pub requested_scope: Vec<String>,
    pub session_id: Option<String>,
}

/// OAuth2 クライアント情報
#[derive(Debug, Deserialize)]
pub struct HydraClientInfo {
    pub client_id: String,
    pub client_name: Option<String>,
}

/// ログイン承認リクエスト（oxgate → Hydra）
#[derive(Debug, Serialize)]
pub struct AcceptLoginRequest {
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remember: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remember_for: Option<i64>,
}

/// ログイン拒否リクエスト（oxgate → Hydra）
#[derive(Debug, Serialize)]
pub struct RejectLoginRequest {
    pub error: String,
    pub error_description: String,
}

/// Hydra リダイレクトレスポンス（Accept/Reject 共通）
#[derive(Debug, Deserialize)]
pub struct HydraRedirectResponse {
    pub redirect_to: String,
}

// ============================================================================
// Consent (同意) フロー関連 DTO
// ============================================================================

/// Hydra からの同意リクエスト情報
#[derive(Debug, Deserialize)]
pub struct HydraConsentRequest {
    pub challenge: String,
    pub skip: bool,
    pub subject: String,
    pub client: HydraClientInfo,
    pub requested_scope: Vec<String>,
    pub requested_access_token_audience: Vec<String>,
}

/// 同意承認リクエスト（oxgate → Hydra）
#[derive(Debug, Serialize)]
pub struct AcceptConsentRequest {
    pub grant_scope: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub grant_access_token_audience: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remember: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remember_for: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<ConsentSession>,
}

/// 同意セッション情報（ID トークンに含めるクレームなど）
#[derive(Debug, Serialize, Default)]
pub struct ConsentSession {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub access_token: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub id_token: HashMap<String, serde_json::Value>,
}

/// 同意拒否リクエスト（oxgate → Hydra）
#[derive(Debug, Serialize)]
pub struct RejectConsentRequest {
    pub error: String,
    pub error_description: String,
}

// ============================================================================
// Logout (ログアウト) フロー関連 DTO
// ============================================================================

/// Hydra からのログアウトリクエスト情報
#[derive(Debug, Deserialize)]
pub struct HydraLogoutRequest {
    pub challenge: String,
    pub subject: String,
    pub sid: Option<String>,
}

/// ログアウト承認リクエスト（oxgate → Hydra）
/// Hydra API は空オブジェクト {} を期待する
#[derive(Debug, Serialize)]
pub struct AcceptLogoutRequest {}

/// ログアウト拒否リクエスト（oxgate → Hydra）
#[derive(Debug, Serialize)]
pub struct RejectLogoutRequest {
    pub error: String,
    pub error_description: String,
}

use crate::error::AppError;

/// Hydra Admin API クライアント
#[derive(Clone)]
pub struct HydraClient {
    client: reqwest::Client,
    admin_url: String,
}

impl HydraClient {
    /// 新しい HydraClient を作成
    pub fn new(admin_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            admin_url,
        }
    }

    /// ログインリクエスト情報を取得
    ///
    /// Hydra Admin API からログインチャレンジの詳細を取得する
    pub async fn get_login_request(&self, challenge: &str) -> Result<HydraLoginRequest, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/login?login_challenge={}",
            self.admin_url, challenge
        );

        let response: reqwest::Response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra login request 取得失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra returned status: {}",
                status
            )));
        }

        let login_request: HydraLoginRequest = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::debug!("Hydra login request 取得成功");
        Ok(login_request)
    }

    /// ログインを承認
    ///
    /// ユーザー認証成功時に Hydra に通知する
    pub async fn accept_login(
        &self,
        challenge: &str,
        subject: &str,
        remember: bool,
        remember_for: i64,
    ) -> Result<String, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/login/accept?login_challenge={}",
            self.admin_url, challenge
        );

        let body = AcceptLoginRequest {
            subject: subject.to_string(),
            remember: Some(remember),
            remember_for: Some(remember_for),
        };

        let response: reqwest::Response = self.client.put(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra accept login 失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra accept returned status: {}",
                status
            )));
        }

        let redirect: HydraRedirectResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::info!("Hydra login accept 成功");
        Ok(redirect.redirect_to)
    }

    /// ログインを拒否
    ///
    /// ユーザー認証失敗時に Hydra に通知する
    pub async fn reject_login(
        &self,
        challenge: &str,
        error: &str,
        description: &str,
    ) -> Result<String, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/login/reject?login_challenge={}",
            self.admin_url, challenge
        );

        let body = RejectLoginRequest {
            error: error.to_string(),
            error_description: description.to_string(),
        };

        let response: reqwest::Response = self.client.put(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra reject login 失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra reject returned status: {}",
                status
            )));
        }

        let redirect: HydraRedirectResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::info!("Hydra login reject 成功");
        Ok(redirect.redirect_to)
    }

    // ========================================================================
    // Consent (同意) フロー関連メソッド
    // ========================================================================

    /// 同意リクエスト情報を取得
    ///
    /// Hydra Admin API から同意チャレンジの詳細を取得する
    pub async fn get_consent_request(
        &self,
        challenge: &str,
    ) -> Result<HydraConsentRequest, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/consent?consent_challenge={}",
            self.admin_url, challenge
        );

        let response: reqwest::Response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra consent request 取得失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra returned status: {}",
                status
            )));
        }

        let consent_request: HydraConsentRequest = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::debug!("Hydra consent request 取得成功");
        Ok(consent_request)
    }

    /// 同意を承認
    ///
    /// ユーザーがスコープを許可した時に Hydra に通知する
    pub async fn accept_consent(
        &self,
        challenge: &str,
        grant_scope: Vec<String>,
        grant_access_token_audience: Vec<String>,
        remember: bool,
        remember_for: i64,
        session: Option<ConsentSession>,
    ) -> Result<String, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/consent/accept?consent_challenge={}",
            self.admin_url, challenge
        );

        let body = AcceptConsentRequest {
            grant_scope,
            grant_access_token_audience,
            remember: Some(remember),
            remember_for: Some(remember_for),
            session,
        };

        let response: reqwest::Response = self.client.put(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra accept consent 失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra accept returned status: {}",
                status
            )));
        }

        let redirect: HydraRedirectResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::info!("Hydra consent accept 成功");
        Ok(redirect.redirect_to)
    }

    /// 同意を拒否
    ///
    /// ユーザーがスコープを拒否した時に Hydra に通知する
    pub async fn reject_consent(
        &self,
        challenge: &str,
        error: &str,
        description: &str,
    ) -> Result<String, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/consent/reject?consent_challenge={}",
            self.admin_url, challenge
        );

        let body = RejectConsentRequest {
            error: error.to_string(),
            error_description: description.to_string(),
        };

        let response: reqwest::Response = self.client.put(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra reject consent 失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra reject returned status: {}",
                status
            )));
        }

        let redirect: HydraRedirectResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::info!("Hydra consent reject 成功");
        Ok(redirect.redirect_to)
    }

    // ========================================================================
    // Logout (ログアウト) フロー関連メソッド
    // ========================================================================

    /// ログアウトリクエスト情報を取得
    ///
    /// Hydra Admin API からログアウトチャレンジの詳細を取得する
    pub async fn get_logout_request(
        &self,
        challenge: &str,
    ) -> Result<HydraLogoutRequest, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/logout?logout_challenge={}",
            self.admin_url, challenge
        );

        let response: reqwest::Response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra logout request 取得失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra returned status: {}",
                status
            )));
        }

        let logout_request: HydraLogoutRequest = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::debug!("Hydra logout request 取得成功");
        Ok(logout_request)
    }

    /// ログアウトを承認
    ///
    /// ユーザーがログアウトを確認した時に Hydra に通知する
    pub async fn accept_logout(&self, challenge: &str) -> Result<String, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/logout/accept?logout_challenge={}",
            self.admin_url, challenge
        );

        let body = AcceptLogoutRequest {};

        let response: reqwest::Response = self.client.put(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra accept logout 失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra accept returned status: {}",
                status
            )));
        }

        let redirect: HydraRedirectResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::info!("Hydra logout accept 成功");
        Ok(redirect.redirect_to)
    }

    /// ログアウトを拒否
    ///
    /// ユーザーがログアウトをキャンセルした時に Hydra に通知する
    pub async fn reject_logout(
        &self,
        challenge: &str,
        error: &str,
        description: &str,
    ) -> Result<String, AppError> {
        let url = format!(
            "{}/admin/oauth2/auth/requests/logout/reject?logout_challenge={}",
            self.admin_url, challenge
        );

        let body = RejectLogoutRequest {
            error: error.to_string(),
            error_description: description.to_string(),
        };

        let response: reqwest::Response = self.client.put(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Hydra reject logout 失敗");
            return Err(AppError::Internal(anyhow::anyhow!(
                "Hydra reject returned status: {}",
                status
            )));
        }

        let redirect: HydraRedirectResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Hydra レスポンスのパースエラー");
            AppError::Internal(anyhow::anyhow!("Failed to parse Hydra response"))
        })?;

        tracing::info!("Hydra logout reject 成功");
        Ok(redirect.redirect_to)
    }
}
