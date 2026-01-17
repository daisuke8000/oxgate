use std::sync::Arc;

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Google OAuth URLs
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

/// OAuth ユーザー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

/// OAuth トークンレスポンス
#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
}

/// Google トークンエンドポイントからのレスポンス
#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: Option<i64>,
}

/// Google userinfo エンドポイントからのレスポンス
#[derive(Debug, Deserialize)]
struct GoogleUserInfoResponse {
    id: String,
    email: String,
    name: Option<String>,
}

/// Google OAuth サービス
///
/// # Security
/// - client_secret はログに出力しない
/// - state パラメータは AES-256-GCM で暗号化
/// - login_challenge を state に埋め込み CSRF 対策
#[derive(Clone)]
pub struct OAuthService {
    client_id: String,
    /// クライアントシークレット（機密情報 - ログ出力禁止）
    client_secret: Arc<String>,
    redirect_uri: String,
    state_encryption_key: [u8; 32],
    http_client: reqwest::Client,
}

impl OAuthService {
    /// 新しい OAuthService を作成
    ///
    /// # Arguments
    /// * `client_id` - Google OAuth クライアントID
    /// * `client_secret` - Google OAuth クライアントシークレット（機密情報）
    /// * `redirect_uri` - OAuth コールバック URI
    /// * `state_secret_base64` - Base64エンコードされた32バイトの暗号化キー
    ///
    /// # Security
    /// `client_secret` は機密情報のため、ログ出力禁止
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        state_secret_base64: &str,
    ) -> Result<Self, AppError> {
        let key_bytes = URL_SAFE_NO_PAD
            .decode(state_secret_base64)
            .or_else(|_| {
                // URL_SAFE でデコード失敗した場合、STANDARD を試す
                base64::engine::general_purpose::STANDARD.decode(state_secret_base64)
            })
            .map_err(|e| {
                tracing::error!(error = ?e, "OAuth state暗号化キーのBase64デコードエラー");
                AppError::Internal(anyhow::anyhow!("invalid state encryption key format"))
            })?;

        if key_bytes.len() != 32 {
            tracing::error!(
                expected = 32,
                actual = key_bytes.len(),
                "OAuth state暗号化キーの長さが不正"
            );
            return Err(AppError::Internal(anyhow::anyhow!(
                "state encryption key must be 32 bytes"
            )));
        }

        let mut state_encryption_key = [0u8; 32];
        state_encryption_key.copy_from_slice(&key_bytes);

        Ok(Self {
            client_id,
            client_secret: Arc::new(client_secret),
            redirect_uri,
            state_encryption_key,
            http_client: reqwest::Client::new(),
        })
    }

    /// Google OAuth 認可 URL を生成
    ///
    /// # Arguments
    /// * `login_challenge` - Hydra から受け取った login_challenge
    ///
    /// # Returns
    /// Google OAuth 認可 URL（state に login_challenge を暗号化して埋め込み）
    pub fn generate_auth_url(&self, login_challenge: &str) -> Result<String, AppError> {
        // login_challenge を暗号化して state に埋め込む
        let encrypted_state = self.encrypt_state(login_challenge)?;

        let params = [
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", "openid email profile"),
            ("state", &encrypted_state),
            ("access_type", "online"),
            ("prompt", "select_account"),
        ];

        let url = reqwest::Url::parse_with_params(GOOGLE_AUTH_URL, &params).map_err(|e| {
            tracing::error!(error = ?e, "OAuth認可URL生成エラー");
            AppError::Internal(anyhow::anyhow!("failed to generate auth url"))
        })?;

        Ok(url.to_string())
    }

    /// 認可コードをアクセストークンに交換
    ///
    /// # Arguments
    /// * `code` - Google から受け取った認可コード
    pub async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, AppError> {
        // application/x-www-form-urlencoded 形式で body を構築
        let body = format!(
            "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(self.client_secret.as_str()),
            urlencoding::encode(code),
            urlencoding::encode(&self.redirect_uri),
        );

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Googleトークンエンドポイント通信エラー");
                AppError::OAuthProviderError
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(
                status = %status,
                body = %body,
                "Googleトークン交換エラー"
            );
            return Err(AppError::OAuthError(format!(
                "token exchange failed: {}",
                status
            )));
        }

        let token_response: GoogleTokenResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Googleトークンレスポンスのパースエラー");
            AppError::OAuthError("invalid token response".to_string())
        })?;

        Ok(OAuthTokenResponse {
            access_token: token_response.access_token,
        })
    }

    /// アクセストークンを使用してユーザー情報を取得
    ///
    /// # Arguments
    /// * `access_token` - Google アクセストークン
    pub async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, AppError> {
        let response = self
            .http_client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Google userinfo API通信エラー");
                AppError::OAuthProviderError
            })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "Google userinfo取得エラー");
            return Err(AppError::OAuthError(format!(
                "userinfo request failed: {}",
                status
            )));
        }

        let user_info: GoogleUserInfoResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "Google userinfoレスポンスのパースエラー");
            AppError::OAuthError("invalid userinfo response".to_string())
        })?;

        Ok(OAuthUserInfo {
            id: user_info.id,
            email: user_info.email,
            name: user_info.name,
        })
    }

    /// state パラメータをデコードして login_challenge を復元
    ///
    /// # Arguments
    /// * `state` - コールバックで受け取った state パラメータ
    ///
    /// # Returns
    /// 復号された login_challenge
    pub fn decode_state(&self, state: &str) -> Result<String, AppError> {
        self.decrypt_state(state)
    }

    /// login_challenge を AES-256-GCM で暗号化し、Base64 URL-safe エンコード
    fn encrypt_state(&self, login_challenge: &str) -> Result<String, AppError> {
        let cipher = Aes256Gcm::new_from_slice(&self.state_encryption_key).map_err(|e| {
            tracing::error!(error = ?e, "AES-GCM暗号化器の初期化エラー");
            AppError::Internal(anyhow::anyhow!("cipher initialization error"))
        })?;

        // 96ビット (12バイト) のランダム nonce 生成
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, login_challenge.as_bytes())
            .map_err(|e| {
                tracing::error!(error = ?e, "state暗号化エラー");
                AppError::Internal(anyhow::anyhow!("state encryption error"))
            })?;

        // nonce + ciphertext を結合して Base64 URL-safe エンコード
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(URL_SAFE_NO_PAD.encode(&combined))
    }

    /// 暗号化された state を復号して login_challenge を取得
    fn decrypt_state(&self, encrypted_state: &str) -> Result<String, AppError> {
        let encrypted = URL_SAFE_NO_PAD.decode(encrypted_state).map_err(|e| {
            tracing::warn!(error = ?e, "state Base64デコードエラー（改ざんの可能性）");
            AppError::OAuthStateInvalid
        })?;

        if encrypted.len() < 12 {
            tracing::warn!(
                len = encrypted.len(),
                "暗号化stateが短すぎる（改ざんの可能性）"
            );
            return Err(AppError::OAuthStateInvalid);
        }

        let cipher = Aes256Gcm::new_from_slice(&self.state_encryption_key).map_err(|e| {
            tracing::error!(error = ?e, "AES-GCM暗号化器の初期化エラー");
            AppError::Internal(anyhow::anyhow!("cipher initialization error"))
        })?;

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            tracing::warn!(error = ?e, "state復号エラー（改ざんまたは期限切れの可能性）");
            AppError::OAuthStateInvalid
        })?;

        String::from_utf8(plaintext).map_err(|e| {
            tracing::warn!(error = ?e, "復号stateのUTF-8変換エラー");
            AppError::OAuthStateInvalid
        })
    }
}

// =============================================================================
// GitHub OAuth サービス
// =============================================================================

/// GitHub OAuth URLs
const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USERINFO_URL: &str = "https://api.github.com/user";

/// GitHub トークンエンドポイントからのレスポンス
#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
}

/// GitHub userinfo エンドポイントからのレスポンス
#[derive(Debug, Deserialize)]
struct GitHubUserInfoResponse {
    id: i64,
    email: Option<String>,
    name: Option<String>,
    login: String,
}

/// GitHub OAuth サービス
///
/// # Security
/// - client_secret はログに出力しない
/// - state パラメータは AES-256-GCM で暗号化
/// - login_challenge を state に埋め込み CSRF 対策
#[derive(Clone)]
pub struct GitHubOAuthService {
    client_id: String,
    /// クライアントシークレット（機密情報 - ログ出力禁止）
    client_secret: Arc<String>,
    redirect_uri: String,
    state_encryption_key: [u8; 32],
    http_client: reqwest::Client,
}

impl GitHubOAuthService {
    /// 新しい GitHubOAuthService を作成
    ///
    /// # Arguments
    /// * `client_id` - GitHub OAuth クライアントID
    /// * `client_secret` - GitHub OAuth クライアントシークレット（機密情報）
    /// * `redirect_uri` - OAuth コールバック URI
    /// * `state_secret_base64` - Base64エンコードされた32バイトの暗号化キー
    ///
    /// # Security
    /// `client_secret` は機密情報のため、ログ出力禁止
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        state_secret_base64: &str,
    ) -> Result<Self, AppError> {
        let key_bytes = URL_SAFE_NO_PAD
            .decode(state_secret_base64)
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(state_secret_base64))
            .map_err(|e| {
                tracing::error!(error = ?e, "GitHub OAuth state暗号化キーのBase64デコードエラー");
                AppError::Internal(anyhow::anyhow!("invalid state encryption key format"))
            })?;

        if key_bytes.len() != 32 {
            tracing::error!(
                expected = 32,
                actual = key_bytes.len(),
                "GitHub OAuth state暗号化キーの長さが不正"
            );
            return Err(AppError::Internal(anyhow::anyhow!(
                "state encryption key must be 32 bytes"
            )));
        }

        let mut state_encryption_key = [0u8; 32];
        state_encryption_key.copy_from_slice(&key_bytes);

        Ok(Self {
            client_id,
            client_secret: Arc::new(client_secret),
            redirect_uri,
            state_encryption_key,
            http_client: reqwest::Client::new(),
        })
    }

    /// GitHub OAuth 認可 URL を生成
    ///
    /// # Arguments
    /// * `login_challenge` - Hydra から受け取った login_challenge
    ///
    /// # Returns
    /// GitHub OAuth 認可 URL（state に login_challenge を暗号化して埋め込み）
    pub fn generate_auth_url(&self, login_challenge: &str) -> Result<String, AppError> {
        let encrypted_state = self.encrypt_state(login_challenge)?;

        let params = [
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("scope", "user:email"),
            ("state", &encrypted_state),
        ];

        let url = reqwest::Url::parse_with_params(GITHUB_AUTH_URL, &params).map_err(|e| {
            tracing::error!(error = ?e, "GitHub OAuth認可URL生成エラー");
            AppError::Internal(anyhow::anyhow!("failed to generate auth url"))
        })?;

        Ok(url.to_string())
    }

    /// 認可コードをアクセストークンに交換
    ///
    /// # Arguments
    /// * `code` - GitHub から受け取った認可コード
    pub async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, AppError> {
        let body = format!(
            "client_id={}&client_secret={}&code={}&redirect_uri={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(self.client_secret.as_str()),
            urlencoding::encode(code),
            urlencoding::encode(&self.redirect_uri),
        );

        let response = self
            .http_client
            .post(GITHUB_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "GitHubトークンエンドポイント通信エラー");
                AppError::OAuthProviderError
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(
                status = %status,
                body = %body,
                "GitHubトークン交換エラー"
            );
            return Err(AppError::OAuthError(format!(
                "token exchange failed: {}",
                status
            )));
        }

        let token_response: GitHubTokenResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "GitHubトークンレスポンスのパースエラー");
            AppError::OAuthError("invalid token response".to_string())
        })?;

        Ok(OAuthTokenResponse {
            access_token: token_response.access_token,
        })
    }

    /// アクセストークンを使用してユーザー情報を取得
    ///
    /// # Arguments
    /// * `access_token` - GitHub アクセストークン
    pub async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, AppError> {
        let response = self
            .http_client
            .get(GITHUB_USERINFO_URL)
            .header("User-Agent", "oxgate")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "GitHub userinfo API通信エラー");
                AppError::OAuthProviderError
            })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(status = %status, "GitHub userinfo取得エラー");
            return Err(AppError::OAuthError(format!(
                "userinfo request failed: {}",
                status
            )));
        }

        let user_info: GitHubUserInfoResponse = response.json().await.map_err(|e| {
            tracing::error!(error = ?e, "GitHub userinfoレスポンスのパースエラー");
            AppError::OAuthError("invalid userinfo response".to_string())
        })?;

        // GitHub ではメールが公開されていない場合がある
        // その場合は login (ユーザー名) を使用
        let email = user_info
            .email
            .unwrap_or_else(|| format!("{}@github.local", user_info.login));

        Ok(OAuthUserInfo {
            id: user_info.id.to_string(),
            email,
            name: user_info.name,
        })
    }

    /// state パラメータをデコードして login_challenge を復元
    pub fn decode_state(&self, state: &str) -> Result<String, AppError> {
        self.decrypt_state(state)
    }

    /// login_challenge を AES-256-GCM で暗号化
    fn encrypt_state(&self, login_challenge: &str) -> Result<String, AppError> {
        let cipher = Aes256Gcm::new_from_slice(&self.state_encryption_key).map_err(|e| {
            tracing::error!(error = ?e, "AES-GCM暗号化器の初期化エラー");
            AppError::Internal(anyhow::anyhow!("cipher initialization error"))
        })?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, login_challenge.as_bytes())
            .map_err(|e| {
                tracing::error!(error = ?e, "state暗号化エラー");
                AppError::Internal(anyhow::anyhow!("state encryption error"))
            })?;

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(URL_SAFE_NO_PAD.encode(&combined))
    }

    /// 暗号化された state を復号
    fn decrypt_state(&self, encrypted_state: &str) -> Result<String, AppError> {
        let encrypted = URL_SAFE_NO_PAD.decode(encrypted_state).map_err(|e| {
            tracing::warn!(error = ?e, "state Base64デコードエラー（改ざんの可能性）");
            AppError::OAuthStateInvalid
        })?;

        if encrypted.len() < 12 {
            tracing::warn!(
                len = encrypted.len(),
                "暗号化stateが短すぎる（改ざんの可能性）"
            );
            return Err(AppError::OAuthStateInvalid);
        }

        let cipher = Aes256Gcm::new_from_slice(&self.state_encryption_key).map_err(|e| {
            tracing::error!(error = ?e, "AES-GCM暗号化器の初期化エラー");
            AppError::Internal(anyhow::anyhow!("cipher initialization error"))
        })?;

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            tracing::warn!(error = ?e, "state復号エラー（改ざんまたは期限切れの可能性）");
            AppError::OAuthStateInvalid
        })?;

        String::from_utf8(plaintext).map_err(|e| {
            tracing::warn!(error = ?e, "復号stateのUTF-8変換エラー");
            AppError::OAuthStateInvalid
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;

    fn create_test_service() -> OAuthService {
        let key = [0u8; 32];
        let key_base64 = STANDARD.encode(key);
        OAuthService::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
            "http://localhost:8080/callback".to_string(),
            &key_base64,
        )
        .unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_state() {
        let service = create_test_service();
        let login_challenge = "test-login-challenge-12345";

        let encrypted = service.encrypt_state(login_challenge).unwrap();
        // Base64 URL-safe エンコードされている
        assert!(!encrypted.is_empty());
        assert!(!encrypted.contains('+'));
        assert!(!encrypted.contains('/'));

        let decrypted = service.decrypt_state(&encrypted).unwrap();
        assert_eq!(login_challenge, decrypted);
    }

    #[test]
    fn test_decode_state_alias() {
        let service = create_test_service();
        let login_challenge = "another-challenge";

        let encrypted = service.encrypt_state(login_challenge).unwrap();
        let decrypted = service.decode_state(&encrypted).unwrap();
        assert_eq!(login_challenge, decrypted);
    }

    #[test]
    fn test_decrypt_invalid_state() {
        let service = create_test_service();

        // 無効な Base64
        let result = service.decrypt_state("not-valid-base64!!!");
        assert!(matches!(result, Err(AppError::OAuthStateInvalid)));

        // 短すぎるデータ
        let short_data = URL_SAFE_NO_PAD.encode([0u8; 5]);
        let result = service.decrypt_state(&short_data);
        assert!(matches!(result, Err(AppError::OAuthStateInvalid)));

        // 改ざんされたデータ
        let tampered = URL_SAFE_NO_PAD.encode([0u8; 50]);
        let result = service.decrypt_state(&tampered);
        assert!(matches!(result, Err(AppError::OAuthStateInvalid)));
    }

    #[test]
    fn test_generate_auth_url() {
        let service = create_test_service();
        let login_challenge = "test-challenge";

        let url = service.generate_auth_url(login_challenge).unwrap();

        assert!(url.starts_with(GOOGLE_AUTH_URL));
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("scope=openid+email+profile"));
        assert!(url.contains("state="));
        assert!(url.contains("redirect_uri="));
    }

    #[test]
    fn test_new_with_invalid_key_length() {
        let short_key = STANDARD.encode([0u8; 16]);
        let result = OAuthService::new(
            "client-id".to_string(),
            "secret".to_string(),
            "http://localhost/callback".to_string(),
            &short_key,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_new_with_invalid_base64() {
        let result = OAuthService::new(
            "client-id".to_string(),
            "secret".to_string(),
            "http://localhost/callback".to_string(),
            "not-valid-base64!!!",
        );
        assert!(result.is_err());
    }

    // ==========================================================================
    // GitHub OAuth サービスのテスト
    // ==========================================================================

    fn create_github_test_service() -> GitHubOAuthService {
        let key = [0u8; 32];
        let key_base64 = STANDARD.encode(key);
        GitHubOAuthService::new(
            "github-client-id".to_string(),
            "github-client-secret".to_string(),
            "http://localhost:8080/github/callback".to_string(),
            &key_base64,
        )
        .unwrap()
    }

    #[test]
    fn test_github_encrypt_decrypt_state() {
        let service = create_github_test_service();
        let login_challenge = "github-login-challenge-12345";

        let encrypted = service.encrypt_state(login_challenge).unwrap();
        // Base64 URL-safe エンコードされている
        assert!(!encrypted.is_empty());
        assert!(!encrypted.contains('+'));
        assert!(!encrypted.contains('/'));

        let decrypted = service.decrypt_state(&encrypted).unwrap();
        assert_eq!(login_challenge, decrypted);
    }

    #[test]
    fn test_github_decode_state_alias() {
        let service = create_github_test_service();
        let login_challenge = "github-challenge";

        let encrypted = service.encrypt_state(login_challenge).unwrap();
        let decrypted = service.decode_state(&encrypted).unwrap();
        assert_eq!(login_challenge, decrypted);
    }

    #[test]
    fn test_github_decrypt_invalid_state() {
        let service = create_github_test_service();

        // 無効な Base64
        let result = service.decrypt_state("not-valid-base64!!!");
        assert!(matches!(result, Err(AppError::OAuthStateInvalid)));

        // 短すぎるデータ
        let short_data = URL_SAFE_NO_PAD.encode([0u8; 5]);
        let result = service.decrypt_state(&short_data);
        assert!(matches!(result, Err(AppError::OAuthStateInvalid)));

        // 改ざんされたデータ
        let tampered = URL_SAFE_NO_PAD.encode([0u8; 50]);
        let result = service.decrypt_state(&tampered);
        assert!(matches!(result, Err(AppError::OAuthStateInvalid)));
    }

    #[test]
    fn test_github_generate_auth_url() {
        let service = create_github_test_service();
        let login_challenge = "github-test-challenge";

        let url = service.generate_auth_url(login_challenge).unwrap();

        assert!(url.starts_with(GITHUB_AUTH_URL));
        assert!(url.contains("client_id=github-client-id"));
        assert!(url.contains("scope=user%3Aemail")); // user:email URL encoded
        assert!(url.contains("state="));
        assert!(url.contains("redirect_uri="));
    }

    #[test]
    fn test_github_new_with_invalid_key_length() {
        let short_key = STANDARD.encode([0u8; 16]);
        let result = GitHubOAuthService::new(
            "github-client-id".to_string(),
            "secret".to_string(),
            "http://localhost/callback".to_string(),
            &short_key,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_github_new_with_invalid_base64() {
        let result = GitHubOAuthService::new(
            "github-client-id".to_string(),
            "secret".to_string(),
            "http://localhost/callback".to_string(),
            "not-valid-base64!!!",
        );
        assert!(result.is_err());
    }
}
