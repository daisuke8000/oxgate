import { Page, Locator } from '@playwright/test';

/**
 * ログインページ Page Object Model
 */
export class LoginPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly loginButton: Locator;
  readonly errorMessage: Locator;
  readonly registerLink: Locator;
  readonly passwordResetLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('input[name="email"]');
    this.passwordInput = page.locator('input[name="password"]');
    this.loginButton = page.locator('button[type="submit"]');
    // より具体的なセレクタ（Next.js の route announcer を除外）
    this.errorMessage = page.locator('[role="alert"]:not([id="__next-route-announcer__"])');
    this.registerLink = page.locator('a[href="/register"]');
    this.passwordResetLink = page.locator('a[href*="password-reset"]');
  }

  /**
   * ログインページに遷移
   */
  async goto(loginChallenge: string) {
    await this.page.goto(`/login?login_challenge=${loginChallenge}`);
  }

  /**
   * ログイン実行
   */
  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.loginButton.click();
  }

  /**
   * エラーメッセージ取得
   */
  async getErrorMessage(): Promise<string | null> {
    if (await this.errorMessage.isVisible()) {
      return await this.errorMessage.textContent();
    }
    return null;
  }

  /**
   * パスワードリセットリンクをクリック
   */
  async clickPasswordResetLink() {
    await this.passwordResetLink.click();
  }

  /**
   * 登録リンクをクリック
   */
  async clickRegisterLink() {
    await this.registerLink.click();
  }
}
