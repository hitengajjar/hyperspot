//! Route registration with OperationBuilder for OpenAPI documentation

use crate::domain::Service;
use super::{dto::*, handlers};
use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use modkit::api::OpenApiRegistry;
use std::sync::Arc;

/// Register all REST routes with OpenAPI documentation
pub fn register_routes(
    router: Router,
    _openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> anyhow::Result<Router> {
    // TODO: Register OpenAPI schemas when OpenApiRegistry API is available
    // For now, utoipa ToSchema derives are sufficient for schema generation

    // Build router with all endpoints
    let router = router
        // Settings endpoints
        .route("/settings", get(list_settings_handler))
        .route("/settings/:setting_type", get(get_setting_handler))
        .route("/settings/:setting_type", put(upsert_setting_handler))
        .route("/settings/:setting_type", delete(delete_setting_handler))
        .route("/settings/:setting_type/lock", put(lock_setting_handler))
        // GTS Type endpoints
        .route("/cti-types", get(list_gts_types_handler))
        .route("/cti-types", post(create_gts_type_handler))
        .route("/cti-types/:type_id", get(get_gts_type_handler))
        .route("/cti-types/:type_id", put(update_gts_type_handler))
        .route("/cti-types/:type_id", delete(delete_gts_type_handler))
        // Add service as extension for handlers
        .layer(Extension(service));

    Ok(router)
}

// ===== Handler wrappers that extract service from Extension =====

async fn list_settings_handler(
    Extension(service): Extension<Arc<Service>>,
    query: axum::extract::Query<handlers::ListSettingsQuery>,
) -> Result<axum::Json<SettingsListResponse>, super::error::Problem> {
    handlers::list_settings(service, query).await
}

async fn get_setting_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
    query: axum::extract::Query<handlers::GetSettingQuery>,
) -> Result<axum::Json<SettingDto>, super::error::Problem> {
    handlers::get_setting(service, path, query).await
}

async fn upsert_setting_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
    json: axum::Json<UpdateSettingRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<SettingDto>), super::error::Problem> {
    handlers::upsert_setting(service, path, json).await
}

async fn delete_setting_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
    query: axum::extract::Query<handlers::GetSettingQuery>,
) -> Result<axum::http::StatusCode, super::error::Problem> {
    handlers::delete_setting(service, path, query).await
}

async fn lock_setting_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
    json: axum::Json<LockSettingRequest>,
) -> Result<axum::http::StatusCode, super::error::Problem> {
    handlers::lock_setting(service, path, json).await
}

async fn list_gts_types_handler(
    Extension(service): Extension<Arc<Service>>,
) -> Result<axum::Json<GtsTypesListResponse>, super::error::Problem> {
    handlers::list_gts_types(service).await
}

async fn get_gts_type_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
) -> Result<axum::Json<GtsTypeDto>, super::error::Problem> {
    handlers::get_gts_type(service, path).await
}

async fn create_gts_type_handler(
    Extension(service): Extension<Arc<Service>>,
    json: axum::Json<UpsertGtsTypeRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<GtsTypeDto>), super::error::Problem> {
    handlers::create_gts_type(service, json).await
}

async fn update_gts_type_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
    json: axum::Json<UpsertGtsTypeRequest>,
) -> Result<axum::Json<GtsTypeDto>, super::error::Problem> {
    handlers::update_gts_type(service, path, json).await
}

async fn delete_gts_type_handler(
    Extension(service): Extension<Arc<Service>>,
    path: axum::extract::Path<String>,
) -> Result<axum::http::StatusCode, super::error::Problem> {
    handlers::delete_gts_type(service, path).await
}
