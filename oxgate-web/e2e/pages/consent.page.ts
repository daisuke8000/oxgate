import { Page, Locator } from '@playwright/test';

/**
 * 同意ページ Page Object Model
 */
export class ConsentPage {
  readonly page: Page;
  readonly acceptButton: Locator;
  readonly rejectButton: Locator;
  readonly scopesList: Locator;

  constructor(page: Page) {
    this.page = page;
    this.acceptButton = page.locator('button:has-text("許可")');
    this.rejectButton = page.locator('button:has-text("拒否")');
    this.scopesList = page.locator('ul li');
  }

  async goto(consentChallenge: string) {
    await this.page.goto(`/consent?consent_challenge=${consentChallenge}`);
  }

  async acceptConsent() {
    await this.acceptButton.click();
  }

  async rejectConsent() {
    await this.rejectButton.click();
  }

  async getScopes(): Promise<string[]> {
    return await this.scopesList.allTextContents();
  }
}
