use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// パスワードリセットトークン
///
/// トークン自体はハッシュ化してDBに保存（token_hash）
/// 平文トークンはユーザーにメールで送信し、DBには保存しない
#[derive(Debug, FromRow, Serialize)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(skip)]
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub used_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}
