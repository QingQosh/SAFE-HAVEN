use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;

/// Current (latest) API version. Bump when introducing breaking changes.
pub const CURRENT_API_VERSION: &str = "v2";

/// Oldest API version still supported. At most two versions run simultaneously.
pub const MIN_SUPPORTED_API_VERSION: &str = "v1";

/// Deprecation timeline for the oldest supported version.
pub const DEPRECATION_POLICY: &str = "API versions are URL-based (/api/{version}/...). At most two versions are supported at once. When a new major version is released, the previous version is marked deprecated and remains available for 6 months before removal. Deprecated responses include Deprecation and Sunset headers.";

/// All API versions that can currently be served.
pub const SUPPORTED_API_VERSIONS: &[&str] = &[MIN_SUPPORTED_API_VERSION, CURRENT_API_VERSION];

#[derive(Serialize)]
struct VersionInfo {
    version: String,
    current: String,
    supported: Vec<String>,
    deprecated: bool,
    deprecation_policy: String,
}

fn is_supported(version: &str) -> bool {
    SUPPORTED_API_VERSIONS.contains(&version)
}

/// Resolve the requested API version from the URL path segment, falling back
/// to the `Accept-Version` header, then to the current version.
fn resolve_version(path_version: Option<&str>, headers: &HeaderMap) -> Option<String> {
    if let Some(v) = path_version {
        return Some(v.to_string());
    }
    if let Some(v) = headers
        .get("accept-version")
        .and_then(|h| h.to_str().ok())
    {
        return Some(v.to_string());
    }
    Some(CURRENT_API_VERSION.to_string())
}

fn version_response(version: &str, body: impl IntoResponse) -> Response {
    let mut response = body.into_response();
    let headers = response.headers_mut();
    if let Ok(v) = header::HeaderValue::from_str(version) {
        headers.insert("x-api-version", v);
    }
    if version != CURRENT_API_VERSION {
        headers.insert("deprecation", header::HeaderValue::from_static("true"));
        headers.insert(
            "sunset",
            header::HeaderValue::from_static("6 months after next major release"),
        );
    }
    response
}

async fn version_info(Path(version): Path<String>) -> Response {
    if !is_supported(&version) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "unsupported_api_version",
                "requested": version,
                "supported": SUPPORTED_API_VERSIONS,
            })),
        )
            .into_response();
    }

    let info = VersionInfo {
        version: version.clone(),
        current: CURRENT_API_VERSION.to_string(),
        supported: SUPPORTED_API_VERSIONS.iter().map(|v| v.to_string()).collect(),
        deprecated: version != CURRENT_API_VERSION,
        deprecation_policy: DEPRECATION_POLICY.to_string(),
    };
    version_response(&version, Json(info))
}

async fn health(Path(version): Path<String>) -> Response {
    if !is_supported(&version) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "unsupported_api_version",
                "requested": version,
                "supported": SUPPORTED_API_VERSIONS,
            })),
        )
            .into_response();
    }
    version_response(
        &version,
        Json(serde_json::json!({
            "status": "ok",
            "version": version,
        })),
    )
}

/// Build the versioned API router. Each supported version is mounted under
/// `/api/{version}` so multiple versions can run simultaneously.
pub fn api_router() -> Router {
    let mut router = Router::new();
    for version in SUPPORTED_API_VERSIONS {
        let version_router = Router::new()
            .route("/health", get(health))
            .route("/version", get(version_info));
        router = router.nest(&format!("/api/{}", version), version_router);
    }
    router
}

/// Resolve the effective API version for a request, honoring the URL path
/// segment first and the `Accept-Version` header second.
pub fn requested_version(path_version: Option<&str>, headers: &HeaderMap) -> Option<String> {
    resolve_version(path_version, headers)
}
