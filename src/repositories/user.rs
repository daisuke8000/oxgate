use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// メールアドレスでユーザーを検索
    ///
    /// # Note
    /// DB セットアップ後は `query_as!` マクロに変更してコンパイル時SQL検証を有効にすること
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    /// ユーザーIDでユーザーを検索
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 新しいユーザーを作成
    ///
    /// # Errors
    /// - UNIQUE制約違反時: `sqlx::Error::Database` (constraint = "users_email_key")
    ///   呼び出し側で `AppError::EmailAlreadyExists` に変換すること
    pub async fn create_user(&self, email: &str, password_hash: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
    }

    /// ユーザーのパスワードを更新
    ///
    /// # Note
    /// password_hash はログに出力しないこと
    pub async fn update_password(
        &self,
        user_id: Uuid,
        new_password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(new_password_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// ソーシャルログイン用ユーザーを作成（パスワードなし）
    ///
    /// # Note
    /// ソーシャルログインのみで登録するユーザー用
    /// 後からパスワードを設定することも可能
    pub async fn create_social_user(&self, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, NULL)
            RETURNING id, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }
}
