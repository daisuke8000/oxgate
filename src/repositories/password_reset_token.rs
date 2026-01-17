use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::PasswordResetToken;

#[derive(Clone)]
pub struct PasswordResetTokenRepository {
    pool: PgPool,
}

impl PasswordResetTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 新しいパスワードリセットトークンを作成
    ///
    /// # Arguments
    /// * `user_id` - 対象ユーザーのID
    /// * `token_hash` - トークンのSHA256ハッシュ
    /// * `expires_at` - 有効期限
    pub async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: OffsetDateTime,
    ) -> Result<PasswordResetToken, sqlx::Error> {
        sqlx::query_as::<_, PasswordResetToken>(
            r#"
            INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, expires_at, used_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    /// トークンハッシュでトークンを検索
    ///
    /// # Note
    /// 有効期限や使用済みフラグの検証は呼び出し側で行う
    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, sqlx::Error> {
        sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT id, user_id, token_hash, expires_at, used_at, created_at
            FROM password_reset_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
    }

    /// トークンを使用済みにマーク
    ///
    /// used_at を現在時刻に設定
    pub async fn mark_as_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 期限切れトークンを削除
    ///
    /// # Returns
    /// 削除された行数
    pub async fn delete_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
