-- user_2fa_secrets テーブル作成
-- ユーザーの二要素認証（TOTP）シークレットを格納
-- シークレットは AES-256-GCM で暗号化して保存

CREATE TABLE user_2fa_secrets (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret_encrypted BYTEA NOT NULL,
    enabled BOOLEAN DEFAULT false NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
