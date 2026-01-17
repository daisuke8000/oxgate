-- password_reset_tokens テーブル作成
-- パスワードリセット用の一時トークンを格納

CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- user_id 検索用インデックス（ユーザーのトークン一覧取得）
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);

-- token_hash のユニークインデックス（トークン検索 + 重複防止）
CREATE UNIQUE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);
