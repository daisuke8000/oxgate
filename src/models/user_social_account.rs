use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// ソーシャルログインプロバイダとユーザーの紐付け
#[derive(Debug, FromRow, Serialize)]
pub struct UserSocialAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_id: String,
    pub email: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
