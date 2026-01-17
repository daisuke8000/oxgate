use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::error::AppError;
use crate::models::User;
use crate::repositories::UserRepository;

/// パスワードをargon2idでハッシュ化
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!(error = ?e, "パスワードハッシュ生成エラー");
            AppError::Internal(anyhow::anyhow!("password hash error"))
        })?;
    Ok(hash.to_string())
}

/// 認証サービス
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    /// 新しい AuthService を作成
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    /// ユーザー認証を実行
    ///
    /// タイミング攻撃対策: ユーザーが存在しない場合もダミーのパスワード検証を実行
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<User, AppError> {
        let user = self.user_repo.find_by_email(email).await?;

        match user {
            Some(user) => {
                // ソーシャルログインユーザー（パスワードなし）の場合は認証失敗
                let password_hash = match &user.password_hash {
                    Some(hash) => hash,
                    None => {
                        // タイミング攻撃対策: ダミーのパスワード検証を実行
                        let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RWh6";
                        let _ = self.verify_password(password, dummy_hash);
                        tracing::warn!(email = %email, "認証失敗: ソーシャルログインユーザー");
                        return Err(AppError::Authentication("invalid_credentials".to_string()));
                    }
                };

                if self.verify_password(password, password_hash)? {
                    tracing::info!(email = %email, "認証成功");
                    Ok(user)
                } else {
                    tracing::warn!(email = %email, "認証失敗: パスワード不一致");
                    Err(AppError::Authentication("invalid_credentials".to_string()))
                }
            }
            None => {
                // タイミング攻撃対策: ユーザーが存在しない場合もダミーのパスワード検証を実行
                // これにより、ユーザーの存在有無を応答時間から推測できなくなる
                let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RWh6";
                let _ = self.verify_password(password, dummy_hash);
                tracing::warn!(email = %email, "認証失敗: ユーザー不在");
                Err(AppError::Authentication("invalid_credentials".to_string()))
            }
        }
    }

    /// パスワードを検証
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            tracing::error!(error = ?e, "パスワードハッシュのパースエラー");
            AppError::Internal(anyhow::anyhow!("password hash parse error"))
        })?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    /// パスワード検証ロジックのユニットテスト
    /// AuthService のインスタンス化には PgPool が必要なため、
    /// argon2 を直接テスト
    #[test]
    fn test_verify_password_logic() {
        // argon2 のパスワード検証ロジックをテスト
        // 無効なハッシュ形式でエラーハンドリングを確認
        let invalid_hash = "invalid_hash_format";
        let parsed = argon2::PasswordHash::new(invalid_hash);
        assert!(parsed.is_err());
    }
}
