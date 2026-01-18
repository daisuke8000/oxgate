/**
 * サンプルクライアント セットアップスクリプト
 *
 * 1. Hydra にクライアント登録
 * 2. テストユーザー作成
 * 3. config.json 生成
 */

const HYDRA_ADMIN_URL = "http://localhost:4445";
const OXGATE_API_URL = "http://localhost:8080";
const REDIRECT_URI = "http://localhost:9000/callback";

async function main() {
  console.log("🚀 サンプルクライアント セットアップ\n");

  // 1. Hydra 起動確認
  console.log("1️⃣  Hydra 起動確認...");
  try {
    const health = await fetch(`${HYDRA_ADMIN_URL}/health/ready`);
    if (!health.ok) throw new Error("Not ready");
    console.log("   ✅ Hydra OK\n");
  } catch (e) {
    console.error("   ❌ Hydra が起動していません");
    console.error("   → docker-compose up -d を実行してください\n");
    process.exit(1);
  }

  // 2. oxgate-api 起動確認
  console.log("2️⃣  oxgate-api 起動確認...");
  try {
    const health = await fetch(`${OXGATE_API_URL}/api/health`);
    if (!health.ok) throw new Error("Not ready");
    console.log("   ✅ oxgate-api OK\n");
  } catch (e) {
    console.error("   ❌ oxgate-api が起動していません");
    console.error("   → docker-compose up -d を実行してください\n");
    process.exit(1);
  }

  // 3. 既存クライアント削除
  console.log("3️⃣  既存クライアント確認...");
  try {
    const clients = await fetch(`${HYDRA_ADMIN_URL}/admin/clients`).then(r => r.json());
    for (const client of clients) {
      if (client.client_id?.startsWith("sample-client-")) {
        await fetch(`${HYDRA_ADMIN_URL}/admin/clients/${client.client_id}`, { method: "DELETE" });
        console.log(`   🗑️  削除: ${client.client_id}`);
      }
    }
  } catch (e) {
    // 無視
  }
  console.log("   ✅ 完了\n");

  // 4. クライアント登録
  console.log("4️⃣  OAuth2 クライアント登録...");
  const clientId = `sample-client-${Date.now()}`;

  const response = await fetch(`${HYDRA_ADMIN_URL}/admin/clients`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      client_id: clientId,
      client_name: "Sample Client App",
      grant_types: ["authorization_code", "refresh_token"],
      response_types: ["code"],
      scope: "openid profile email",
      redirect_uris: [REDIRECT_URI],
      token_endpoint_auth_method: "client_secret_post",
    }),
  });

  const client = await response.json();

  if (!response.ok) {
    console.error("   ❌ クライアント登録失敗:", client);
    process.exit(1);
  }

  console.log(`   ✅ Client ID: ${client.client_id}`);
  console.log(`   ✅ Client Secret: ${client.client_secret}\n`);

  // 5. テストユーザー作成
  console.log("5️⃣  テストユーザー作成...");
  const userResponse = await fetch(`${OXGATE_API_URL}/api/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      email: "test@example.com",
      password: "password123",
    }),
  });

  if (userResponse.ok) {
    console.log("   ✅ テストユーザー作成完了");
  } else {
    const err = await userResponse.json();
    if (err.error === "email_already_exists") {
      console.log("   ✅ テストユーザーは既に存在します");
    } else {
      console.log(`   ⚠️  ${JSON.stringify(err)}`);
    }
  }
  console.log("   📧 Email: test@example.com");
  console.log("   🔑 Password: password123\n");

  // 6. config.json 生成
  console.log("6️⃣  config.json 生成...");
  const config = {
    clientId: client.client_id,
    clientSecret: client.client_secret,
    hydraPublicUrl: "http://localhost:4444",
    redirectUri: REDIRECT_URI,
    scopes: "openid profile email",
  };

  await Bun.write("./config.json", JSON.stringify(config, null, 2));
  console.log("   ✅ config.json を生成しました\n");

  // 完了
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("✨ セットアップ完了！");
  console.log("");
  console.log("次のコマンドでサンプルアプリを起動:");
  console.log("  bun run dev");
  console.log("");
  console.log("ブラウザで http://localhost:9000 にアクセス");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

main().catch(console.error);
