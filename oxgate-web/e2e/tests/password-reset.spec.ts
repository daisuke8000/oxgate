import { test, expect } from '@playwright/test';
import { PasswordResetRequestPage, PasswordResetConfirmPage } from '../pages/password-reset.page';
import { RESET_USER } from '../fixtures/test-data';

test.describe('パスワードリセット機能', () => {
  test.describe('リセット要求', () => {
    let requestPage: PasswordResetRequestPage;

    test.beforeEach(async ({ page }) => {
      requestPage = new PasswordResetRequestPage(page);
    });

    test('正常なリセット要求フロー', async ({ page }) => {
      // APIモックを設定
      await page.route('**/api/password-reset/request', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            message: 'パスワードリセット用のメールを送信しました',
          }),
        });
      });

      await requestPage.goto();

      // リセット要求
      await requestPage.requestReset(RESET_USER.email);

      // 成功メッセージ確認
      const successCard = page.locator('text=送信完了');
      await expect(successCard).toBeVisible({ timeout: 5000 });
    });

    test('存在しないメールアドレスでも成功メッセージ表示 (セキュリティ対策)', async ({ page }) => {
      // APIモックを設定 (常に200を返す)
      await page.route('**/api/password-reset/request', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            message: 'パスワードリセット用のメールを送信しました',
          }),
        });
      });

      await requestPage.goto();
      await requestPage.requestReset('nonexistent@example.com');

      // タイミング攻撃対策のため、常に成功メッセージを表示
      const successCard = page.locator('text=送信完了');
      await expect(successCard).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe('リセット確認', () => {
    let confirmPage: PasswordResetConfirmPage;

    test.beforeEach(async ({ page }) => {
      confirmPage = new PasswordResetConfirmPage(page);
    });

    test('正常なパスワード再設定フロー', async ({ page }) => {
      // APIモックを設定
      await page.route('**/api/password-reset/confirm', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            message: 'パスワードが再設定されました',
          }),
        });
      });

      const validToken = 'valid-reset-token-12345';
      await confirmPage.goto(validToken);

      // 新しいパスワードを設定
      await confirmPage.confirmReset(
        RESET_USER.newPassword,
        RESET_USER.newPassword
      );

      // 成功メッセージ確認
      const successCard = page.locator('text=パスワード変更完了');
      await expect(successCard).toBeVisible({ timeout: 5000 });
    });

    test('無効なトークンでエラー表示', async ({ page }) => {
      // APIモックを設定 (無効なトークン)
      await page.route('**/api/password-reset/confirm', async (route) => {
        await route.fulfill({
          status: 400,
          contentType: 'application/json',
          body: JSON.stringify({
            error: 'invalid_token',
            message: 'トークンが無効または期限切れです',
          }),
        });
      });

      await confirmPage.goto('invalid-token');
      await confirmPage.confirmReset(
        RESET_USER.newPassword,
        RESET_USER.newPassword
      );

      // エラーメッセージ確認
      const errorMessage = await confirmPage.getErrorMessage();
      expect(errorMessage).toContain('トークンが無効');
    });

    test('パスワード不一致エラー', async ({ page }) => {
      await confirmPage.goto('valid-token');

      // 異なるパスワードを入力
      await confirmPage.confirmReset(
        RESET_USER.newPassword,
        'DifferentPassword123!'
      );

      // エラーメッセージ確認（Zod バリデーション）
      const errorText = page.locator('text=パスワードが一致しません');
      await expect(errorText).toBeVisible();
    });

    test('トークンパラメータがない場合のエラー', async ({ page }) => {
      await page.goto('/password-reset/confirm');

      // エラーメッセージが表示されることを確認（Next.js route announcer を除外）
      const errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
      await expect(errorMessage).toBeVisible();
      await expect(errorMessage).toContainText('token');
    });
  });
});
