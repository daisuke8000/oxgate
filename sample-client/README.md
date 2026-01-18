# oxgate サンプルクライアント

OAuth2 認証フローをテストするための軽量クライアントアプリ（Hono + Bun）。

## 起動方法

```bash
# 1. oxgate サービス起動（プロジェクトルートで）
docker-compose up -d

# 2. セットアップ（Hydra にクライアント登録）
cd sample-client
bun install
bun run setup

# 3. サンプルアプリ起動
bun run dev
```

## 画面フロー

```
http://localhost:9000          [ログイン] ボタン
        ↓
http://localhost:4444/...      Hydra（自動リダイレクト）
        ↓
http://localhost:3000/login    oxgate ログイン画面
        ↓
http://localhost:3000/consent  oxgate 同意画面
        ↓
http://localhost:9000/callback コールバック（トークン交換）
        ↓
http://localhost:9000/dashboard ダッシュボード（ログイン完了）
```

## テストアカウント

| 項目 | 値 |
|------|-----|
| Email | test@example.com |
| Password | password123 |

## ファイル構成

```
sample-client/
├── server.ts    # Hono サーバー
├── setup.ts     # セットアップスクリプト
├── config.json  # OAuth2 設定（setup で生成）
└── package.json # hono 依存
```
