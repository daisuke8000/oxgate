import { Page, Locator } from '@playwright/test';

/**
 * ログアウトページ Page Object Model
 */
export class LogoutPage {
  readonly page: Page;
  readonly confirmButton: Locator;
  readonly cancelButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.confirmButton = page.locator('button:has-text("ログアウト")');
    this.cancelButton = page.locator('button:has-text("キャンセル")');
  }

  async goto(logoutChallenge: string) {
    await this.page.goto(`/logout?logout_challenge=${logoutChallenge}`);
  }

  async confirmLogout() {
    await this.confirmButton.click();
  }

  async cancelLogout() {
    await this.cancelButton.click();
  }
}
