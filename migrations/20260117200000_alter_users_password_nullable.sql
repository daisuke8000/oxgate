-- users テーブルの password_hash を NULL 許可に変更
-- ソーシャルログインのみで登録したユーザーはパスワードを持たない

ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;
