import { test, expect } from '@playwright/test';
import { TwoFactorPage } from '../pages/two-factor.page';
import { TEST_USER, TOTP_TEST } from '../fixtures/test-data';

test.describe('二要素認証機能', () => {
  let twoFactorPage: TwoFactorPage;

  test.beforeEach(async ({ page }) => {
    twoFactorPage = new TwoFactorPage(page);
  });

  test('2FA有効化フロー', async ({ page }) => {
    // Setup APIモック
    await page.route('**/api/2fa/setup', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          secret: TOTP_TEST.secret,
          qr_code: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
        }),
      });
    });

    // Verify APIモック
    await page.route('**/api/2fa/verify', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          enabled: true,
        }),
      });
    });

    await twoFactorPage.goto();

    // セットアップ開始
    await twoFactorPage.startSetup();

    // パスワード入力
    await twoFactorPage.enterPassword(TEST_USER.password);

    // QRコード表示確認
    const qrSrc = await twoFactorPage.getQrCodeSrc();
    expect(qrSrc).toContain('data:image');

    // シークレット表示確認
    const secret = await twoFactorPage.getSecret();
    expect(secret).toBeTruthy();

    // TOTP コード検証
    await twoFactorPage.verifyCode(TOTP_TEST.validCode);

    // 成功メッセージ確認
    const successText = page.locator('text=2FA有効');
    await expect(successText).toBeVisible({ timeout: 5000 });
  });

  test('無効なTOTPコードでエラー表示', async ({ page }) => {
    // Setup APIモック
    await page.route('**/api/2fa/setup', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          secret: TOTP_TEST.secret,
          qr_code: 'data:image/png;base64,dummy',
        }),
      });
    });

    // Verify APIモック (エラー)
    await page.route('**/api/2fa/verify', async (route) => {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'invalid_code',
          message: '認証コードが正しくありません',
        }),
      });
    });

    await twoFactorPage.goto();
    await twoFactorPage.startSetup();
    await twoFactorPage.enterPassword(TEST_USER.password);

    // 無効なコードを入力
    await twoFactorPage.verifyCode(TOTP_TEST.invalidCode);

    // エラーメッセージ確認
    const errorMessage = await twoFactorPage.getErrorMessage();
    expect(errorMessage).toContain('認証コードが正しくありません');
  });

  test('誤ったパスワードでエラー表示', async ({ page }) => {
    // Setup APIモック (パスワードエラー)
    await page.route('**/api/2fa/setup', async (route) => {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'invalid_credentials',
          message: 'パスワードが正しくありません',
        }),
      });
    });

    await twoFactorPage.goto();
    await twoFactorPage.startSetup();
    await twoFactorPage.enterPassword('wrongpassword');

    // エラーメッセージ確認
    const errorMessage = await twoFactorPage.getErrorMessage();
    expect(errorMessage).toContain('パスワードが正しくありません');
  });

  test('2FA無効化フロー', async ({ page }) => {
    // Disable APIモック
    await page.route('**/api/2fa/disable', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          disabled: true,
        }),
      });
    });

    await twoFactorPage.goto();

    // まず2FAを有効化した状態をシミュレート（stepを変更）
    // 実際のテストでは状態管理が必要
    // ここでは無効化ボタンが表示される状態を仮定
  });

  test('2FA既に有効化済みエラー', async ({ page }) => {
    // Setup APIモック (既に有効)
    await page.route('**/api/2fa/setup', async (route) => {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'already_enabled',
          message: '二要素認証は既に有効化されています',
        }),
      });
    });

    await twoFactorPage.goto();
    await twoFactorPage.startSetup();
    await twoFactorPage.enterPassword(TEST_USER.password);

    // エラーメッセージ確認
    const errorMessage = await twoFactorPage.getErrorMessage();
    expect(errorMessage).toContain('既に有効化されています');
  });
});
