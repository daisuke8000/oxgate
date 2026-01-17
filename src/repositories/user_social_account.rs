use sqlx::PgPool;
use uuid::Uuid;

use crate::models::UserSocialAccount;

#[derive(Clone)]
pub struct UserSocialAccountRepository {
    pool: PgPool,
}

impl UserSocialAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// プロバイダとプロバイダIDでソーシャルアカウントを検索
    ///
    /// # Note
    /// ソーシャルログイン時に既存ユーザーを特定するために使用
    pub async fn find_by_provider_and_id(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> Result<Option<UserSocialAccount>, sqlx::Error> {
        sqlx::query_as::<_, UserSocialAccount>(
            r#"
            SELECT id, user_id, provider, provider_id, email, created_at, updated_at
            FROM user_social_accounts
            WHERE provider = $1 AND provider_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// ユーザーIDに紐付くソーシャルアカウント一覧を取得
    pub async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSocialAccount>, sqlx::Error> {
        sqlx::query_as::<_, UserSocialAccount>(
            r#"
            SELECT id, user_id, provider, provider_id, email, created_at, updated_at
            FROM user_social_accounts
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// 新しいソーシャルアカウント紐付けを作成
    ///
    /// # Errors
    /// - UNIQUE制約違反時: 同一プロバイダ・プロバイダIDの組み合わせが既に存在
    pub async fn create(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_id: &str,
        email: Option<&str>,
    ) -> Result<UserSocialAccount, sqlx::Error> {
        sqlx::query_as::<_, UserSocialAccount>(
            r#"
            INSERT INTO user_social_accounts (user_id, provider, provider_id, email)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, provider, provider_id, email, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_id)
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }
}
