import { Page, Locator } from '@playwright/test';

/**
 * パスワードリセット要求ページ Page Object Model
 */
export class PasswordResetRequestPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly submitButton: Locator;
  readonly successMessage: Locator;
  readonly loginLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('input[name="email"]');
    this.submitButton = page.locator('button[type="submit"]');
    this.successMessage = page.locator('text=送信完了');
    this.loginLink = page.locator('a[href="/login"]');
  }

  async goto() {
    await this.page.goto('/password-reset/request');
  }

  async requestReset(email: string) {
    await this.emailInput.fill(email);
    await this.submitButton.click();
  }

  async isSuccess(): Promise<boolean> {
    return await this.successMessage.isVisible();
  }
}

/**
 * パスワードリセット確認ページ Page Object Model
 */
export class PasswordResetConfirmPage {
  readonly page: Page;
  readonly newPasswordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly submitButton: Locator;
  readonly errorMessage: Locator;
  readonly successMessage: Locator;
  readonly loginLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.newPasswordInput = page.locator('input[name="newPassword"]');
    this.confirmPasswordInput = page.locator('input[name="confirmPassword"]');
    this.submitButton = page.locator('button[type="submit"]');
    // より具体的なセレクタ（Next.js の route announcer を除外）
    this.errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
    this.successMessage = page.locator('text=パスワード変更完了');
    this.loginLink = page.locator('a[href="/login"]');
  }

  async goto(token: string) {
    await this.page.goto(`/password-reset/confirm?token=${token}`);
  }

  async confirmReset(newPassword: string, confirmPassword: string) {
    await this.newPasswordInput.fill(newPassword);
    await this.confirmPasswordInput.fill(confirmPassword);
    await this.submitButton.click();
  }

  async isSuccess(): Promise<boolean> {
    return await this.successMessage.isVisible();
  }

  async getErrorMessage(): Promise<string | null> {
    if (await this.errorMessage.isVisible()) {
      return await this.errorMessage.textContent();
    }
    return null;
  }
}
