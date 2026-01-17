import { Page, Locator } from '@playwright/test';

/**
 * 二要素認証設定ページ Page Object Model
 */
export class TwoFactorPage {
  readonly page: Page;
  readonly enableButton: Locator;
  readonly passwordInput: Locator;
  readonly setupButton: Locator;
  readonly qrCode: Locator;
  readonly secretText: Locator;
  readonly codeInput: Locator;
  readonly verifyButton: Locator;
  readonly disableButton: Locator;
  readonly disablePasswordInput: Locator;
  readonly disableCodeInput: Locator;
  readonly disableConfirmButton: Locator;
  readonly successMessage: Locator;
  readonly errorMessage: Locator;

  constructor(page: Page) {
    this.page = page;
    this.enableButton = page.locator('button:has-text("2FAを有効にする")');
    this.passwordInput = page.locator('input[type="password"]').first();
    this.setupButton = page.locator('button:has-text("次へ")');
    this.qrCode = page.locator('img[alt="QR Code"]');
    this.secretText = page.locator('code');
    this.codeInput = page.locator('input[id="code"]');
    this.verifyButton = page.locator('button:has-text("確認")');
    this.disableButton = page.locator('button:has-text("2FAを無効にする")');
    this.disablePasswordInput = page.locator('input[id="disable-password"]');
    this.disableCodeInput = page.locator('input[id="disable-code"]');
    this.disableConfirmButton = page.locator('button:has-text("無効にする")');
    this.successMessage = page.locator('text=2FA有効');
    // より具体的なセレクタ（Next.js の route announcer を除外）
    this.errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
  }

  async goto() {
    await this.page.goto('/settings/2fa');
  }

  async startSetup() {
    await this.enableButton.click();
  }

  async enterPassword(password: string) {
    await this.passwordInput.fill(password);
    await this.setupButton.click();
  }

  async getQrCodeSrc(): Promise<string | null> {
    return await this.qrCode.getAttribute('src');
  }

  async getSecret(): Promise<string | null> {
    return await this.secretText.textContent();
  }

  async verifyCode(code: string) {
    await this.codeInput.fill(code);
    await this.verifyButton.click();
  }

  async isSetupSuccess(): Promise<boolean> {
    return await this.successMessage.isVisible();
  }

  async startDisable() {
    await this.disableButton.click();
  }

  async disable(password: string, code: string) {
    await this.disablePasswordInput.fill(password);
    await this.disableCodeInput.fill(code);
    await this.disableConfirmButton.click();
  }

  async getErrorMessage(): Promise<string | null> {
    if (await this.errorMessage.isVisible()) {
      return await this.errorMessage.textContent();
    }
    return null;
  }
}
