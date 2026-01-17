import { test, expect } from '@playwright/test';
import { RegisterPage } from '../pages/register.page';
import { NEW_USER } from '../fixtures/test-data';

test.describe('ユーザー登録機能', () => {
  let registerPage: RegisterPage;

  test.beforeEach(async ({ page }) => {
    registerPage = new RegisterPage(page);
  });

  test('正常な登録フロー', async ({ page }) => {
    // APIモックを設定
    await page.route('**/api/register', async (route) => {
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          message: '登録が完了しました',
        }),
      });
    });

    await registerPage.goto();

    // フォーム入力
    await registerPage.register(
      NEW_USER.email,
      NEW_USER.password,
      NEW_USER.password
    );

    // 成功メッセージ確認
    const successCard = page.locator('text=登録完了');
    await expect(successCard).toBeVisible({ timeout: 5000 });
  });

  test('パスワード不一致エラー', async ({ page }) => {
    await registerPage.goto();

    // 異なるパスワードを入力
    await registerPage.register(
      NEW_USER.email,
      NEW_USER.password,
      'DifferentPassword123!'
    );

    // エラーメッセージ確認（Zod バリデーション）
    const errorText = page.locator('text=パスワードが一致しません');
    await expect(errorText).toBeVisible();
  });

  test('重複メールアドレスエラー', async ({ page }) => {
    // APIモックを設定 (重複エラー)
    await page.route('**/api/register', async (route) => {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'duplicate_email',
          message: 'このメールアドレスは既に登録されています',
        }),
      });
    });

    await registerPage.goto();
    await registerPage.register(
      'existing@example.com',
      NEW_USER.password,
      NEW_USER.password
    );

    // エラーメッセージ確認
    const errorMessage = await registerPage.getErrorMessage();
    expect(errorMessage).toContain('既に登録されています');
  });

  test('バリデーションエラー (短いパスワード)', async ({ page }) => {
    await registerPage.goto();

    // 短いパスワードを入力
    await registerPage.register(
      NEW_USER.email,
      'short',
      'short'
    );

    // エラーメッセージ確認 (クライアント側バリデーション)
    const errorText = page.locator('text=8文字以上');
    await expect(errorText).toBeVisible();
  });

  test('ログインリンクの遷移', async ({ page }) => {
    await registerPage.goto();

    // ログインリンクをクリック
    await registerPage.clickLoginLink();

    // ログインページに遷移
    await expect(page).toHaveURL(/login/);
  });
});
