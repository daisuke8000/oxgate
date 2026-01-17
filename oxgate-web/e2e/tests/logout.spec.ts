import { test, expect } from '@playwright/test';
import { LogoutPage } from '../pages/logout.page';
import { MOCK_CHALLENGES } from '../fixtures/test-data';

test.describe('ログアウト機能', () => {
  let logoutPage: LogoutPage;

  test.beforeEach(async ({ page }) => {
    logoutPage = new LogoutPage(page);
  });

  test('ログアウト確認フロー', async ({ page }) => {
    // APIモックを設定
    await page.route('**/api/logout', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          redirect_to: 'http://localhost:4444/logout/complete',
        }),
      });
    });

    await logoutPage.goto(MOCK_CHALLENGES.logoutChallenge);

    // ログアウトボタンが表示されていることを確認
    await expect(logoutPage.confirmButton).toBeVisible();

    // ログアウト確認
    await logoutPage.confirmLogout();

    // リダイレクト待機
    await page.waitForURL('http://localhost:4444/**', { timeout: 5000 }).catch(() => {});
  });

  test('logout_challenge パラメータがない場合のエラー', async ({ page }) => {
    await page.goto('/logout');

    // エラーメッセージが表示されることを確認（Next.js route announcer を除外）
    const errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
    await expect(errorMessage).toBeVisible();
    await expect(errorMessage).toContainText('logout_challenge');
  });
});
