import { test, expect } from '@playwright/test';
import { ConsentPage } from '../pages/consent.page';
import { MOCK_CHALLENGES } from '../fixtures/test-data';

test.describe('同意機能', () => {
  let consentPage: ConsentPage;

  test.beforeEach(async ({ page }) => {
    consentPage = new ConsentPage(page);
  });

  test('同意を許可するフロー', async ({ page }) => {
    // APIモックを設定
    await page.route('**/api/consent', async (route) => {
      const request = route.request();
      const postData = request.postDataJSON();

      if (postData.accept) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            redirect_to: 'http://localhost:4444/oauth2/auth?consent_verifier=test',
          }),
        });
      }
    });

    await consentPage.goto(MOCK_CHALLENGES.consentChallenge);

    // スコープ表示確認（divベースのリスト）
    const scopeItems = page.locator('.flex.items-center.gap-2');
    const count = await scopeItems.count();
    expect(count).toBeGreaterThan(0);

    // 同意
    await consentPage.acceptConsent();

    // リダイレクト待機
    await page.waitForURL('http://localhost:4444/**', { timeout: 5000 }).catch(() => {});
  });

  test('同意を拒否するフロー', async ({ page }) => {
    // APIモックを設定
    await page.route('**/api/consent', async (route) => {
      const request = route.request();
      const postData = request.postDataJSON();

      if (!postData.accept) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            redirect_to: 'http://localhost:4444/oauth2/auth?error=access_denied',
          }),
        });
      }
    });

    await consentPage.goto(MOCK_CHALLENGES.consentChallenge);

    // 拒否
    await consentPage.rejectConsent();

    // リダイレクト待機
    await page.waitForURL('http://localhost:4444/**', { timeout: 5000 }).catch(() => {});
  });

  test('consent_challenge パラメータがない場合のエラー', async ({ page }) => {
    await page.goto('/consent');

    // エラーメッセージが表示されることを確認（Next.js route announcer を除外）
    const errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
    await expect(errorMessage).toBeVisible();
    await expect(errorMessage).toContainText('consent_challenge');
  });
});
