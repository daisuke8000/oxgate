# oxgate-web E2E テスト

Playwright を使用した包括的なエンドツーエンドテスト。

## 📋 目次

- [テスト構成](#テスト構成)
- [セットアップ](#セットアップ)
- [テスト実行](#テスト実行)
- [テストシナリオ](#テストシナリオ)
- [Page Object Model](#page-object-model)
- [CI/CD統合](#cicd統合)

## テスト構成

```
e2e/
├── fixtures/
│   └── test-data.ts          # テストデータ定義
├── pages/                     # Page Object Models
│   ├── login.page.ts
│   ├── consent.page.ts
│   ├── logout.page.ts
│   ├── register.page.ts
│   ├── password-reset.page.ts
│   └── two-factor.page.ts
├── tests/                     # テストスペック
│   ├── login.spec.ts
│   ├── consent.spec.ts
│   ├── logout.spec.ts
│   ├── register.spec.ts
│   ├── password-reset.spec.ts
│   └── two-factor.spec.ts
└── README.md
```

## セットアップ

### 前提条件

- Bun 1.x以降
- Node.js 18以降 (Playwrightブラウザ用)

### インストール

```bash
cd oxgate-web

# 依存関係インストール
bun install

# Playwrightブラウザインストール
bunx playwright install chromium
```

## テスト実行

### 基本コマンド

```bash
# 全テスト実行 (ヘッドレスモード)
bun run test:e2e

# UIモードで実行 (インタラクティブ)
bun run test:e2e:ui

# ブラウザ表示モードで実行
bun run test:e2e:headed

# デバッグモード
bun run test:e2e:debug

# レポート表示
bun run test:e2e:report
```

### 特定のテストファイル実行

```bash
# ログインテストのみ実行
bunx playwright test login.spec.ts

# 2FAテストのみ実行
bunx playwright test two-factor.spec.ts
```

### 並列実行制御

```bash
# ワーカー数指定
bunx playwright test --workers=2

# シリアル実行
bunx playwright test --workers=1
```

## テストシナリオ

### 1. ログインフロー (login.spec.ts)

| テストケース | 説明 |
|------------|------|
| 正常なログインフロー | 有効な認証情報でログイン成功 |
| 無効な認証情報でエラー表示 | 誤ったパスワードでエラー |
| login_challenge パラメータなしエラー | 必須パラメータチェック |
| パスワードリセットリンク遷移 | ナビゲーション確認 |
| 登録リンク遷移 | ナビゲーション確認 |

### 2. 同意フロー (consent.spec.ts)

| テストケース | 説明 |
|------------|------|
| 同意を許可するフロー | スコープ許可とリダイレクト |
| 同意を拒否するフロー | アクセス拒否処理 |
| consent_challenge パラメータなしエラー | 必須パラメータチェック |

### 3. ログアウトフロー (logout.spec.ts)

| テストケース | 説明 |
|------------|------|
| ログアウト確認フロー | ログアウト実行 |
| ログアウトキャンセルフロー | キャンセル処理 |
| logout_challenge パラメータなしエラー | 必須パラメータチェック |

### 4. ユーザー登録 (register.spec.ts)

| テストケース | 説明 |
|------------|------|
| 正常な登録フロー | 新規ユーザー作成成功 |
| パスワード不一致エラー | 確認パスワードチェック |
| 重複メールアドレスエラー | 既存ユーザーチェック |
| バリデーションエラー | 入力値検証 |
| ログインリンク遷移 | 登録後のナビゲーション |

### 5. パスワードリセット (password-reset.spec.ts)

| テストケース | 説明 |
|------------|------|
| 正常なリセット要求フロー | メール送信リクエスト |
| 存在しないメールでも成功表示 | タイミング攻撃対策 |
| 正常なパスワード再設定フロー | トークン検証と再設定 |
| 無効なトークンでエラー | トークン有効期限チェック |
| パスワード不一致エラー | 確認パスワードチェック |
| トークンパラメータなしエラー | 必須パラメータチェック |

### 6. 二要素認証 (two-factor.spec.ts)

| テストケース | 説明 |
|------------|------|
| 2FA有効化フロー | QRコード表示とTOTP検証 |
| 無効なTOTPコードでエラー | コード検証失敗 |
| 誤ったパスワードでエラー | パスワード確認 |
| 2FA無効化フロー | 2FA解除処理 |
| 2FA既に有効化済みエラー | 重複有効化防止 |

## Page Object Model

### 設計原則

1. **カプセル化**: ページ要素とアクションをクラスにカプセル化
2. **再利用性**: 共通操作をメソッド化
3. **保守性**: UI変更時はPage Objectのみ修正
4. **可読性**: テストコードがシナリオを表現

### 例: LoginPage

```typescript
export class LoginPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly loginButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('input[name="email"]');
    this.passwordInput = page.locator('input[name="password"]');
    this.loginButton = page.locator('button[type="submit"]');
  }

  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.loginButton.click();
  }
}
```

### テストでの使用

```typescript
test('正常なログイン', async ({ page }) => {
  const loginPage = new LoginPage(page);
  await loginPage.goto(MOCK_CHALLENGES.loginChallenge);
  await loginPage.login(TEST_USER.email, TEST_USER.password);
  // アサーション...
});
```

## モック戦略

### APIモックパターン

テストでは `page.route()` を使用してAPIレスポンスをモック:

```typescript
await page.route('**/api/login', async (route) => {
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      redirect_to: 'http://localhost:4444/oauth2/auth?login_verifier=test',
    }),
  });
});
```

### モック vs 実際のバックエンド

#### モードA: モックモード (デフォルト)
- APIレスポンスを `page.route()` でモック
- バックエンドサーバー不要
- 高速実行
- フロントエンドのみのテスト

#### モードB: 統合モード
- 実際のバックエンドAPIを使用
- Docker Composeで全サービス起動
- 完全なE2Eテスト
- 環境変数で制御可能

```bash
# 統合モード実行例
PLAYWRIGHT_BASE_URL=http://localhost:3000 \
INTEGRATION_MODE=true \
bunx playwright test
```

## CI/CD統合

### GitHub Actions 例

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
      
      - name: Install dependencies
        run: |
          cd oxgate-web
          bun install
      
      - name: Install Playwright browsers
        run: bunx playwright install --with-deps chromium
      
      - name: Run E2E tests
        run: bun run test:e2e
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 30
```

### Docker環境でのテスト

```bash
# Docker Composeで全サービス起動
docker-compose up -d

# テスト実行
cd oxgate-web
PLAYWRIGHT_BASE_URL=http://localhost:3000 bun run test:e2e

# サービス停止
docker-compose down
```

## トラブルシューティング

### テスト失敗時のデバッグ

1. **スクリーンショット確認**
   - `test-results/` ディレクトリに自動保存

2. **ビデオ再生**
   - 失敗時のみ録画 (`retain-on-failure`)

3. **トレース確認**
   ```bash
   bunx playwright show-trace test-results/.../.../trace.zip
   ```

4. **UIモードでデバッグ**
   ```bash
   bun run test:e2e:ui
   ```

### よくある問題

#### タイムアウトエラー

```typescript
// タイムアウト延長
test('時間のかかるテスト', async ({ page }) => {
  test.setTimeout(60000); // 60秒
  // ...
});
```

#### セレクタが見つからない

```typescript
// 待機を追加
await page.waitForSelector('button[type="submit"]');
await page.click('button[type="submit"]');
```

#### フラグな (不安定な) テスト

```typescript
// リトライ設定
test.describe.configure({ retries: 2 });
```

## ベストプラクティス

### 1. テストの独立性
- 各テストは独立して実行可能にする
- テスト間で状態を共有しない

### 2. 明確なアサーション
```typescript
// ❌ 悪い例
expect(await page.locator('.error').isVisible()).toBe(true);

// ✅ 良い例
await expect(page.locator('.error')).toBeVisible();
```

### 3. 適切な待機
```typescript
// ❌ 悪い例
await page.waitForTimeout(5000); // 固定時間待機

// ✅ 良い例
await page.waitForURL('**/success');
await expect(page.locator('.success')).toBeVisible();
```

### 4. Page Object の活用
- ページ固有のロジックはPage Objectに集約
- テストコードはビジネスロジックに集中

### 5. テストデータの管理
- `fixtures/test-data.ts` で一元管理
- マジックナンバーを避ける

## 参考資料

- [Playwright 公式ドキュメント](https://playwright.dev/)
- [Page Object Model](https://playwright.dev/docs/pom)
- [Best Practices](https://playwright.dev/docs/best-practices)
- [CI/CD Integration](https://playwright.dev/docs/ci)
