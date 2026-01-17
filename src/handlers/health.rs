use axum::Json;
use serde::Serialize;

/// ヘルスチェックレスポンス
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

/// ヘルスチェックハンドラー
///
/// GET /api/health
///
/// サービスの稼働状況を返す。
/// ロードバランサーやモニタリングツールから呼び出される。
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let response = health_check().await;
        assert_eq!(response.status, "ok");
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
    }
}
