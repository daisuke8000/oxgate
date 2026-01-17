use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
