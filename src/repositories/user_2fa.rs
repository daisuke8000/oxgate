use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User2faSecret;

#[derive(Clone)]
pub struct User2faSecretRepository {
    pool: PgPool,
}

impl User2faSecretRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// ユーザーIDで2FAシークレットを検索
    pub async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<User2faSecret>, sqlx::Error> {
        sqlx::query_as::<_, User2faSecret>(
            r#"
            SELECT user_id, secret_encrypted, enabled, created_at, updated_at
            FROM user_2fa_secrets
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 新しい2FAシークレットを作成
    ///
    /// # Note
    /// 作成時は enabled = false
    /// verify 成功後に enable() を呼び出す
    pub async fn create(
        &self,
        user_id: Uuid,
        secret_encrypted: &[u8],
    ) -> Result<User2faSecret, sqlx::Error> {
        sqlx::query_as::<_, User2faSecret>(
            r#"
            INSERT INTO user_2fa_secrets (user_id, secret_encrypted)
            VALUES ($1, $2)
            RETURNING user_id, secret_encrypted, enabled, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(secret_encrypted)
        .fetch_one(&self.pool)
        .await
    }

    /// 2FAを有効化
    pub async fn enable(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE user_2fa_secrets
            SET enabled = true, updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 2FAを無効化
    pub async fn disable(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE user_2fa_secrets
            SET enabled = false, updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 2FAシークレットを削除
    pub async fn delete(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM user_2fa_secrets
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
