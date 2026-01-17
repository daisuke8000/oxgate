use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use data_encoding::BASE32;
use rand::RngCore;
use totp_rs::{Algorithm, TOTP};

use crate::error::AppError;

/// TOTP (Time-based One-Time Password) サービス
///
/// # Security
/// - シークレットはAES-256-GCMで暗号化してDB保存
/// - シークレット平文はログに出力しない
#[derive(Clone)]
pub struct TotpService {
    issuer: String,
    encryption_key: [u8; 32],
}

impl TotpService {
    /// 新しい TotpService を作成
    ///
    /// # Arguments
    /// * `issuer` - TOTP発行者名（アプリ名）
    /// * `encryption_key_base64` - Base64エンコードされた32バイトの暗号化キー
    pub fn new(issuer: String, encryption_key_base64: &str) -> Result<Self, AppError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let key_bytes = STANDARD.decode(encryption_key_base64).map_err(|e| {
            tracing::error!(error = ?e, "TOTP暗号化キーのBase64デコードエラー");
            AppError::Internal(anyhow::anyhow!("invalid encryption key format"))
        })?;

        if key_bytes.len() != 32 {
            tracing::error!(
                expected = 32,
                actual = key_bytes.len(),
                "TOTP暗号化キーの長さが不正"
            );
            return Err(AppError::Internal(anyhow::anyhow!(
                "encryption key must be 32 bytes"
            )));
        }

        let mut encryption_key = [0u8; 32];
        encryption_key.copy_from_slice(&key_bytes);

        Ok(Self {
            issuer,
            encryption_key,
        })
    }

    /// 20バイトのランダムシークレットを生成し、Base32でエンコード
    pub fn generate_secret() -> String {
        let mut bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        BASE32.encode(&bytes)
    }

    /// シークレットをAES-256-GCMで暗号化
    ///
    /// # Returns
    /// 96ビットnonce (12バイト) + 暗号文
    pub fn encrypt_secret(&self, secret: &str) -> Result<Vec<u8>, AppError> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key).map_err(|e| {
            tracing::error!(error = ?e, "AES-GCM暗号化器の初期化エラー");
            AppError::Internal(anyhow::anyhow!("cipher initialization error"))
        })?;

        // 96ビット (12バイト) のランダムnonce生成
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, secret.as_bytes()).map_err(|e| {
            tracing::error!(error = ?e, "シークレット暗号化エラー");
            AppError::Internal(anyhow::anyhow!("encryption error"))
        })?;

        // nonce + ciphertext を結合
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// 暗号化されたシークレットを復号
    pub fn decrypt_secret(&self, encrypted: &[u8]) -> Result<String, AppError> {
        if encrypted.len() < 12 {
            tracing::error!(len = encrypted.len(), "暗号化データが短すぎる");
            return Err(AppError::Internal(anyhow::anyhow!(
                "encrypted data too short"
            )));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key).map_err(|e| {
            tracing::error!(error = ?e, "AES-GCM暗号化器の初期化エラー");
            AppError::Internal(anyhow::anyhow!("cipher initialization error"))
        })?;

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            tracing::error!(error = ?e, "シークレット復号エラー");
            AppError::Internal(anyhow::anyhow!("decryption error"))
        })?;

        String::from_utf8(plaintext).map_err(|e| {
            tracing::error!(error = ?e, "復号データのUTF-8変換エラー");
            AppError::Internal(anyhow::anyhow!("invalid utf8 after decryption"))
        })
    }

    /// QRコードを生成（PNG形式、Base64エンコード）
    ///
    /// # Arguments
    /// * `email` - ユーザーのメールアドレス（アカウント識別子）
    /// * `secret` - Base32エンコードされたシークレット
    pub fn generate_qr_code(&self, email: &str, secret: &str) -> Result<String, AppError> {
        let totp = self.create_totp(email, secret)?;

        let qr_code = totp.get_qr_base64().map_err(|e| {
            tracing::error!(error = %e, "QRコード生成エラー");
            AppError::Internal(anyhow::anyhow!("qr code generation error"))
        })?;

        Ok(qr_code)
    }

    /// TOTPコードを検証
    ///
    /// # Note
    /// 前後1ステップの時間ウィンドウを許容（±30秒）
    pub fn verify_code(&self, secret: &str, code: &str) -> Result<bool, AppError> {
        // 入力検証: コードは6桁の数字のみ
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Ok(false);
        }

        let totp = self.create_totp_for_verify(secret)?;

        // 現在時刻でのコード検証（前後1ステップを許容）
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                tracing::error!(error = ?e, "システム時刻取得エラー");
                AppError::Internal(anyhow::anyhow!("system time error"))
            })?
            .as_secs();

        // check_current は内部で skew を考慮して検証
        Ok(totp.check(code, current_time))
    }

    /// TOTP オブジェクトを作成（QRコード生成用）
    fn create_totp(&self, email: &str, secret: &str) -> Result<TOTP, AppError> {
        let secret_bytes = BASE32.decode(secret.as_bytes()).map_err(|e| {
            tracing::error!(error = ?e, "シークレットのBase32デコードエラー");
            AppError::Internal(anyhow::anyhow!("invalid base32 secret"))
        })?;

        TOTP::new(
            Algorithm::SHA1,
            6,  // 6桁
            1,  // skew: 前後1ステップ許容
            30, // period: 30秒
            secret_bytes,
            Some(self.issuer.clone()),
            email.to_string(),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "TOTP作成エラー");
            AppError::Internal(anyhow::anyhow!("totp creation error"))
        })
    }

    /// TOTP オブジェクトを作成（検証用）
    fn create_totp_for_verify(&self, secret: &str) -> Result<TOTP, AppError> {
        let secret_bytes = BASE32.decode(secret.as_bytes()).map_err(|e| {
            tracing::error!(error = ?e, "シークレットのBase32デコードエラー");
            AppError::Internal(anyhow::anyhow!("invalid base32 secret"))
        })?;

        TOTP::new(
            Algorithm::SHA1,
            6,  // 6桁
            1,  // skew: 前後1ステップ許容
            30, // period: 30秒
            secret_bytes,
            None,
            String::new(),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "TOTP作成エラー");
            AppError::Internal(anyhow::anyhow!("totp creation error"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    fn create_test_service() -> TotpService {
        // テスト用の32バイトキー
        let key = [0u8; 32];
        let key_base64 = STANDARD.encode(key);
        TotpService::new("TestApp".to_string(), &key_base64).unwrap()
    }

    #[test]
    fn test_generate_secret() {
        let secret = TotpService::generate_secret();
        // Base32エンコードされた20バイト = 32文字
        assert_eq!(secret.len(), 32);
        // Base32文字のみ
        assert!(
            secret
                .chars()
                .all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".contains(c))
        );
    }

    #[test]
    fn test_encrypt_decrypt_secret() {
        let service = create_test_service();
        let original = TotpService::generate_secret();

        let encrypted = service.encrypt_secret(&original).unwrap();
        // 12バイトnonce + 暗号文 + 16バイトtag
        assert!(encrypted.len() > 12);

        let decrypted = service.decrypt_secret(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_generate_qr_code() {
        let service = create_test_service();
        let secret = TotpService::generate_secret();

        let qr_base64 = service
            .generate_qr_code("test@example.com", &secret)
            .unwrap();
        // Base64エンコードされたPNG
        assert!(!qr_base64.is_empty());
    }

    #[test]
    fn test_verify_invalid_code_format() {
        let service = create_test_service();
        let secret = TotpService::generate_secret();

        // 6桁でない
        assert!(!service.verify_code(&secret, "12345").unwrap());
        // 数字以外を含む
        assert!(!service.verify_code(&secret, "12345a").unwrap());
    }

    #[test]
    fn test_new_with_invalid_key_length() {
        let short_key = STANDARD.encode([0u8; 16]); // 16バイト（短すぎる）
        let result = TotpService::new("TestApp".to_string(), &short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_with_invalid_base64() {
        let result = TotpService::new("TestApp".to_string(), "not-valid-base64!!!");
        assert!(result.is_err());
    }
}
