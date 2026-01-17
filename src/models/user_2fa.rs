use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// ユーザーの二要素認証（TOTP）シークレット
///
/// シークレットは AES-256-GCM で暗号化されて保存される
/// 平文シークレットはログに出力禁止
#[derive(Debug, FromRow, Serialize)]
pub struct User2faSecret {
    pub user_id: Uuid,
    #[serde(skip)]
    pub secret_encrypted: Vec<u8>,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
