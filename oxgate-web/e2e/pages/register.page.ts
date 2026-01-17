import { Page, Locator } from '@playwright/test';

/**
 * 登録ページ Page Object Model
 */
export class RegisterPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly registerButton: Locator;
  readonly errorMessage: Locator;
  readonly successMessage: Locator;
  readonly loginLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('input[name="email"]');
    this.passwordInput = page.locator('input[name="password"]');
    this.confirmPasswordInput = page.locator('input[name="confirmPassword"]');
    this.registerButton = page.locator('button[type="submit"]');
    // より具体的なセレクタ（Next.js の route announcer を除外）
    this.errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
    this.successMessage = page.locator('text=登録完了');
    this.loginLink = page.locator('a[href="/login"]');
  }

  async goto() {
    await this.page.goto('/register');
  }

  async register(email: string, password: string, confirmPassword: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.confirmPasswordInput.fill(confirmPassword);
    await this.registerButton.click();
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

  async clickLoginLink() {
    await this.loginLink.click();
  }
}
