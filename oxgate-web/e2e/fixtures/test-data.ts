/**
 * E2Eテスト用のテストデータ
 */

export const TEST_USER = {
  email: 'test@example.com',
  password: 'TestPassword123!',
  invalidPassword: 'wrongpassword',
};

export const NEW_USER = {
  email: 'newuser@example.com',
  password: 'NewPassword123!',
};

export const RESET_USER = {
  email: 'reset@example.com',
  password: 'OldPassword123!',
  newPassword: 'NewPassword123!',
};

/**
 * OAuth2 チャレンジのモック値
 * 実際のテストではHydraから受け取る値を使用
 */
export const MOCK_CHALLENGES = {
  loginChallenge: 'test-login-challenge-12345',
  consentChallenge: 'test-consent-challenge-12345',
  logoutChallenge: 'test-logout-challenge-12345',
};

/**
 * テスト用のスコープ
 */
export const TEST_SCOPES = ['openid', 'profile', 'email'];

/**
 * テスト用のリダイレクトURL
 */
export const MOCK_REDIRECT_URL = 'http://localhost:4444/oauth2/auth?client_id=test';

/**
 * 2FAテスト用データ
 */
export const TOTP_TEST = {
  secret: 'JBSWY3DPEHPK3PXP',
  validCode: '123456', // モック用 (実際のテストでは動的生成が必要)
  invalidCode: '000000',
};

/**
 * APIエンドポイント
 */
export const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
