import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E テスト設定
 * 参照: https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e/tests',
  
  // 並列実行設定
  fullyParallel: true,
  
  // CI環境での失敗時リトライ
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  
  // 並列ワーカー数
  workers: process.env.CI ? 1 : undefined,
  
  // レポート設定
  reporter: [
    ['html'],
    ['list'],
  ],
  
  // 共通設定
  use: {
    // ベースURL (環境変数で上書き可能)
    baseURL: process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:3000',
    
    // トレース設定 (失敗時のみ)
    trace: 'on-first-retry',
    
    // スクリーンショット設定
    screenshot: 'only-on-failure',
    
    // ビデオ設定
    video: 'retain-on-failure',
    
    // タイムアウト設定
    actionTimeout: 10000,
    navigationTimeout: 30000,
  },
  
  // テストプロジェクト設定
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    
    // 他のブラウザは必要に応じて有効化
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],
  
  // Webサーバー設定 (開発サーバーを自動起動)
  webServer: {
    command: 'bun run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});
