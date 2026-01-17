use std::net::SocketAddr;

use axum::{
    Router,
    routing::{get, post},
};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use oxgate::{config::Config, handlers, services::hydra::HydraClient, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ログ初期化（JSON形式、環境変数でレベル制御）
    init_tracing();

    tracing::info!("oxgate 起動中...");

    // 設定読み込み
    let config = Config::load().map_err(|e| {
        tracing::error!(error = ?e, "設定の読み込みに失敗");
        anyhow::anyhow!("Failed to load config: {}", e)
    })?;

    tracing::info!(host = %config.host, port = %config.port, "設定読み込み完了");

    // サーバーアドレスを先に構築（config が move される前に）
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| {
            tracing::error!(error = ?e, "アドレスのパースに失敗");
            anyhow::anyhow!("Failed to parse address: {}", e)
        })?;

    // データベース接続プール作成
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.database_url.expose_secret())
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "データベース接続に失敗");
            anyhow::anyhow!("Failed to connect to database: {}", e)
        })?;

    tracing::info!("データベース接続完了");

    // Hydra クライアント初期化
    let hydra_client = HydraClient::new(config.hydra_admin_url.clone());

    tracing::info!(hydra_url = %config.hydra_admin_url, "Hydra クライアント初期化完了");

    // AppState 構築
    let state = AppState::new(db_pool, hydra_client, config).map_err(|e| {
        tracing::error!(error = ?e, "AppState の構築に失敗");
        anyhow::anyhow!("Failed to create AppState: {}", e)
    })?;

    // Router 構築
    let app = create_router(state);

    // サーバー起動
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        tracing::error!(error = ?e, addr = %addr, "ポートのバインドに失敗");
        anyhow::anyhow!("Failed to bind to {}: {}", addr, e)
    })?;

    tracing::info!(addr = %addr, "サーバー起動");

    // Graceful shutdown 対応
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "サーバーエラー");
            anyhow::anyhow!("Server error: {}", e)
        })?;

    tracing::info!("サーバー終了");

    Ok(())
}

/// tracing の初期化（JSON形式）
fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,oxgate=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// Router の構築
fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(handlers::health_check))
        .route("/api/login", post(handlers::login))
        .route("/api/consent", post(handlers::consent))
        .route("/api/logout", post(handlers::logout))
        // Phase 4: ユーザー管理
        .route("/api/register", post(handlers::register))
        .route(
            "/api/password/reset-request",
            post(handlers::request_password_reset),
        )
        .route("/api/password/reset", post(handlers::reset_password))
        // Phase 5: 二要素認証
        .route("/api/2fa/setup", post(handlers::setup_2fa))
        .route("/api/2fa/verify", post(handlers::verify_2fa))
        .route("/api/2fa/disable", post(handlers::disable_2fa))
        // Phase 6: ソーシャルログイン
        .route("/api/oauth/google", get(handlers::google_auth))
        .route("/api/oauth/google/callback", get(handlers::google_callback))
        .route("/api/oauth/github", get(handlers::github_auth))
        .route("/api/oauth/github/callback", get(handlers::github_callback))
        .with_state(state)
}

/// Graceful shutdown シグナル待機
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!(error = ?e, "Ctrl+C ハンドラーのインストールに失敗");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                tracing::error!(error = ?e, "SIGTERM ハンドラーのインストールに失敗");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl+C received, starting graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("SIGTERM received, starting graceful shutdown");
        }
    }
}
