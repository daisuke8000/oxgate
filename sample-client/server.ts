import { Hono } from "hono";
import type { HtmlEscapedString } from "hono/utils/html";
import { html, raw } from "hono/html";

const app = new Hono();

// ========== 設定 ==========
const config: {
  clientId: string;
  clientSecret: string;
  hydraPublicUrl: string;
  hydraInternalUrl?: string;
  redirectUri: string;
  scopes: string;
} = {
  clientId: "test-app",
  clientSecret: "",
  hydraPublicUrl: "http://localhost:4444",
  hydraInternalUrl: undefined,
  redirectUri: "http://localhost:9000/callback",
  scopes: "openid profile email",
};

// トークン交換用URL (Docker内部では hydraInternalUrl を使用)
const getTokenUrl = () => config.hydraInternalUrl ?? config.hydraPublicUrl;

// config.json があれば読み込み
const configFile = Bun.file("./config.json");
if (await configFile.exists()) {
  const loaded = await configFile.json();
  Object.assign(config, loaded);
  console.log("✅ config.json loaded");
}

// ========== 共通レイアウト ==========
type HtmlContent = HtmlEscapedString | Promise<HtmlEscapedString>;
const layout = (title: string, content: HtmlContent) => html`
  <!doctype html>
  <html lang="ja">
    <head>
      <meta charset="UTF-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />
      <title>${title}</title>
      <style>
        * {
          box-sizing: border-box;
          margin: 0;
          padding: 0;
        }
        body {
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
          background: #f5f5f5;
          min-height: 100vh;
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 2rem;
        }
        .card {
          background: white;
          padding: 2rem;
          border-radius: 12px;
          box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
          max-width: 600px;
          width: 100%;
        }
        h1 {
          margin-bottom: 1rem;
        }
        .btn {
          display: inline-block;
          padding: 0.75rem 1.5rem;
          background: #0070f3;
          color: white;
          text-decoration: none;
          border-radius: 8px;
          font-weight: 600;
        }
        .btn:hover {
          background: #0051a8;
        }
        .btn-red {
          background: #dc3545;
        }
        .btn-red:hover {
          background: #c82333;
        }
        .info {
          background: #f0f0f0;
          padding: 1rem;
          border-radius: 8px;
          margin: 1rem 0;
        }
        .success {
          background: #d4edda;
          color: #155724;
          padding: 1rem;
          border-radius: 8px;
          margin-bottom: 1rem;
        }
        .error {
          background: #f8d7da;
          color: #721c24;
          padding: 1rem;
          border-radius: 8px;
          margin-bottom: 1rem;
        }
        code {
          background: #e0e0e0;
          padding: 2px 6px;
          border-radius: 4px;
        }
        pre {
          background: #f8f9fa;
          padding: 1rem;
          border-radius: 8px;
          overflow-x: auto;
          font-size: 0.8rem;
        }
        .section {
          margin-bottom: 1.5rem;
        }
        .section h2 {
          font-size: 1rem;
          color: #666;
          margin-bottom: 0.5rem;
        }
      </style>
    </head>
    <body>
      <div class="card">${content}</div>
    </body>
  </html>
`;

// ========== ルート ==========

app.get("/", (c) =>
  c.html(
    layout(
      "サンプルアプリ",
      html`
        <h1>📦 サンプルアプリ</h1>
        <p style="color: #666; margin-bottom: 2rem">
          oxgate OAuth2 認証テスト用
        </p>
        <a href="/login" class="btn">🔐 ログイン</a>
        <div class="info">
          <strong>設定:</strong><br />
          Client ID: <code>${config.clientId}</code><br />
          Scopes: <code>${config.scopes}</code>
        </div>
      `,
    ),
  ),
);

app.get("/login", (c) => {
  const state = `state-${Date.now()}`;
  const params = new URLSearchParams({
    client_id: config.clientId,
    response_type: "code",
    scope: config.scopes,
    redirect_uri: config.redirectUri,
    state,
  });
  return c.redirect(`${config.hydraPublicUrl}/oauth2/auth?${params}`);
});

app.get("/callback", async (c) => {
  const code = c.req.query("code");
  const error = c.req.query("error");
  const errorDesc = c.req.query("error_description");

  if (error) {
    return c.html(
      layout(
        "エラー",
        html`
          <h1>❌ エラー</h1>
          <div class="error">
            <strong>${error}</strong><br />${errorDesc ?? ""}
          </div>
          <a href="/" class="btn">🏠 戻る</a>
        `,
      ),
    );
  }

  if (!code) {
    return c.html(
      layout(
        "エラー",
        html`
          <h1>❌ エラー</h1>
          <div class="error">認可コードがありません</div>
          <a href="/" class="btn">🏠 戻る</a>
        `,
      ),
    );
  }

  try {
    const res = await fetch(`${getTokenUrl()}/oauth2/token`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "authorization_code",
        code,
        redirect_uri: config.redirectUri,
        client_id: config.clientId,
        client_secret: config.clientSecret,
      }),
    });

    const tokens = (await res.json()) as {
      access_token?: string;
      id_token?: string;
      error?: string;
      error_description?: string;
    };

    if (!res.ok) {
      throw new Error(
        tokens.error_description ?? tokens.error ?? "Unknown error",
      );
    }

    // ID トークンをデコード
    let userInfo: Record<string, unknown> = {};
    if (tokens.id_token) {
      try {
        const payload = tokens.id_token.split(".")[1];
        userInfo = JSON.parse(atob(payload));
      } catch {
        // ignore decode error
      }
    }

    return c.html(
      layout(
        "ダッシュボード",
        html`
          <h1>🎉 ログイン成功</h1>
          <div class="success">OAuth2 認証フローが完了しました</div>
          <div class="section">
            <h2>👤 ユーザー情報</h2>
            <pre>${raw(JSON.stringify(userInfo, null, 2))}</pre>
          </div>
          <div class="section">
            <h2>🔑 Access Token</h2>
            <pre>${tokens.access_token ?? "(なし)"}</pre>
          </div>
          <div class="section">
            <h2>🎫 ID Token</h2>
            <pre style="max-height: 150px; overflow-y: auto">
${tokens.id_token ?? "(なし)"}</pre
            >
          </div>
          <a href="/logout" class="btn btn-red">🚪 ログアウト</a>
        `,
      ),
    );
  } catch (e) {
    return c.html(
      layout(
        "エラー",
        html`
          <h1>❌ トークン交換エラー</h1>
          <div class="error">${String(e)}</div>
          <a href="/" class="btn">🏠 戻る</a>
        `,
      ),
    );
  }
});

app.get("/logout", (c) =>
  c.html(
    layout(
      "ログアウト",
      html`
        <h1>👋 ログアウトしました</h1>
        <p style="color: #666; margin: 1rem 0">
          またのご利用をお待ちしています。
        </p>
        <a href="/" class="btn">🏠 トップに戻る</a>
      `,
    ),
  ),
);

// ========== サーバー起動 ==========
const port = 9000;
console.log(`🚀 Sample Client: http://localhost:${port}`);
console.log(`   Client ID: ${config.clientId}`);

export default { port, fetch: app.fetch };
