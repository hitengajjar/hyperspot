//! HTTP request handlers - thin layer that delegates to domain service

use crate::domain::Service;
use super::{dto::*, error::{map_domain_error, Problem}};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

// ===== Settings Handlers =====

/// Query parameters for listing settings
#[derive(Debug, Deserialize)]
pub struct ListSettingsQuery {
    /// Filter by tenant ID
    pub tenant_id: Option<Uuid>,
    /// Filter by GTS type
    pub r#type: Option<String>,
}

/// List settings with optional filters
pub async fn list_settings(
    service: Arc<Service>,
    Query(query): Query<ListSettingsQuery>,
) -> Result<Json<SettingsListResponse>, Problem> {
    let settings = if let Some(setting_type) = query.r#type {
        service
            .get_settings_by_type(&setting_type, query.tenant_id)
            .await
            .map_err(map_domain_error)?
    } else if let Some(_tenant_id) = query.tenant_id {
        // TODO: Implement list by tenant in service
        vec![]
    } else {
        // TODO: Implement list all in service
        vec![]
    };

    let items: Vec<SettingDto> = settings.into_iter().map(|s| s.into()).collect();
    let total = items.len();

    Ok(Json(SettingsListResponse { items, total }))
}

/// Get a specific setting
pub async fn get_setting(
    service: Arc<Service>,
    Path(setting_type): Path<String>,
    Query(query): Query<GetSettingQuery>,
) -> Result<Json<SettingDto>, Problem> {
    let setting = service
        .get_setting(&setting_type, query.tenant_id, &query.domain_object_id)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(setting.into()))
}

#[derive(Debug, Deserialize)]
pub struct GetSettingQuery {
    pub tenant_id: Uuid,
    #[serde(default = "default_domain_object_id")]
    pub domain_object_id: String,
}

fn default_domain_object_id() -> String {
    "generic".to_string()
}

/// Update or create a setting
pub async fn upsert_setting(
    service: Arc<Service>,
    Path(setting_type): Path<String>,
    Json(req): Json<UpdateSettingRequest>,
) -> Result<(StatusCode, Json<SettingDto>), Problem> {
    let setting = service
        .upsert_setting(&setting_type, req.tenant_id, &req.domain_object_id, req.data)
        .await
        .map_err(map_domain_error)?;

    Ok((StatusCode::OK, Json(setting.into())))
}

/// Delete a setting
pub async fn delete_setting(
    service: Arc<Service>,
    Path(setting_type): Path<String>,
    Query(query): Query<GetSettingQuery>,
) -> Result<StatusCode, Problem> {
    service
        .delete_setting(&setting_type, query.tenant_id, &query.domain_object_id)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Lock a setting for compliance mode
pub async fn lock_setting(
    service: Arc<Service>,
    Path(setting_type): Path<String>,
    Json(req): Json<LockSettingRequest>,
) -> Result<StatusCode, Problem> {
    service
        .lock_setting(&setting_type, req.tenant_id, &req.domain_object_id, req.read_only)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

// ===== GTS Type Handlers =====

/// List all GTS types
pub async fn list_gts_types(
    service: Arc<Service>,
) -> Result<Json<GtsTypesListResponse>, Problem> {
    let gts_types = service.list_gts_types().await.map_err(map_domain_error)?;

    let items: Vec<GtsTypeDto> = gts_types.into_iter().map(|ct| ct.into()).collect();
    let total = items.len();

    Ok(Json(GtsTypesListResponse { items, total }))
}

/// Get a specific GTS type
pub async fn get_gts_type(
    service: Arc<Service>,
    Path(type_id): Path<String>,
) -> Result<Json<GtsTypeDto>, Problem> {
    let gts_type = service
        .get_gts_type(&type_id)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(gts_type.into()))
}

/// Create a new GTS type
pub async fn create_gts_type(
    service: Arc<Service>,
    Json(req): Json<UpsertGtsTypeRequest>,
) -> Result<(StatusCode, Json<GtsTypeDto>), Problem> {
    let gts_type = service
        .register_gts_type(req.into())
        .await
        .map_err(map_domain_error)?;

    Ok((StatusCode::CREATED, Json(gts_type.into())))
}

/// Update an existing GTS type
pub async fn update_gts_type(
    service: Arc<Service>,
    Path(_type_id): Path<String>,
    Json(req): Json<UpsertGtsTypeRequest>,
) -> Result<Json<GtsTypeDto>, Problem> {
    let gts_type = service
        .update_gts_type(req.into())
        .await
        .map_err(map_domain_error)?;

    Ok(Json(gts_type.into()))
}

/// Delete a GTS type
pub async fn delete_gts_type(
    service: Arc<Service>,
    Path(type_id): Path<String>,
) -> Result<StatusCode, Problem> {
    service
        .delete_gts_type(&type_id)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}
