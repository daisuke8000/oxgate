-- user_social_accounts テーブル作成
-- ソーシャルログインプロバイダとユーザーの紐付けを格納
-- 1ユーザーに対して複数のプロバイダを紐付け可能

CREATE TABLE user_social_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_id VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE(provider, provider_id)
);

-- user_id 検索用インデックス（ユーザーの紐付け済みプロバイダ一覧取得）
CREATE INDEX idx_user_social_accounts_user_id ON user_social_accounts(user_id);

-- provider 検索用インデックス（プロバイダ別の検索）
CREATE INDEX idx_user_social_accounts_provider ON user_social_accounts(provider);
