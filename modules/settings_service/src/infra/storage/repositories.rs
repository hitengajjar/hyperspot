//! SeaORM repository implementations

use crate::contract::{GtsType, Setting};
use crate::domain::repository::{GtsTypeRepository, SettingsRepository};
use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{
    prelude::Expr, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use std::sync::Arc;
use uuid::Uuid;

use super::entity;

// ===== Settings Repository =====

pub struct SeaOrmSettingsRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmSettingsRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SettingsRepository for SeaOrmSettingsRepository {
    async fn upsert(&self, setting: &Setting) -> Result<Setting> {
        use sea_orm::ActiveValue::Set;
        
        // Try to find existing setting
        let existing = entity::Entity::find()
            .filter(entity::Column::Type.eq(&setting.r#type))
            .filter(entity::Column::TenantId.eq(setting.tenant_id))
            .filter(entity::Column::DomainObjectId.eq(&setting.domain_object_id))
            .one(&*self.db)
            .await?;

        let result = if existing.is_some() {
            // Update existing
            let mut active: entity::ActiveModel = setting.into();
            active.updated_at = Set(chrono::Utc::now().into());
            entity::Entity::update(active).exec(&*self.db).await?
        } else {
            // Insert new
            let active: entity::ActiveModel = setting.into();
            entity::Entity::insert(active)
                .exec_with_returning(&*self.db)
                .await?
        };

        Ok(result.into())
    }

    async fn find_by_key(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<Option<Setting>> {
        let result = entity::Entity::find()
            .filter(entity::Column::Type.eq(setting_type))
            .filter(entity::Column::TenantId.eq(tenant_id))
            .filter(entity::Column::DomainObjectId.eq(domain_object_id))
            .filter(entity::Column::DeletedAt.is_null())
            .one(&*self.db)
            .await?;

        Ok(result.map(|e| e.into()))
    }

    async fn find_by_type(
        &self,
        setting_type: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<Setting>> {
        let mut query = entity::Entity::find()
            .filter(entity::Column::Type.eq(setting_type))
            .filter(entity::Column::DeletedAt.is_null());

        if let Some(tid) = tenant_id {
            query = query.filter(entity::Column::TenantId.eq(tid));
        }

        let results = query
            .order_by_asc(entity::Column::TenantId)
            .order_by_asc(entity::Column::DomainObjectId)
            .all(&*self.db)
            .await?;

        Ok(results.into_iter().map(|e| e.into()).collect())
    }

    async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Setting>> {
        let results = entity::Entity::find()
            .filter(entity::Column::TenantId.eq(tenant_id))
            .filter(entity::Column::DeletedAt.is_null())
            .order_by_asc(entity::Column::Type)
            .all(&*self.db)
            .await?;

        Ok(results.into_iter().map(|e| e.into()).collect())
    }

    async fn find_by_domain_object(&self, domain_object_id: &str) -> Result<Vec<Setting>> {
        let results = entity::Entity::find()
            .filter(entity::Column::DomainObjectId.eq(domain_object_id))
            .filter(entity::Column::DeletedAt.is_null())
            .order_by_asc(entity::Column::Type)
            .all(&*self.db)
            .await?;

        Ok(results.into_iter().map(|e| e.into()).collect())
    }

    async fn soft_delete(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<()> {
        entity::Entity::update_many()
            .col_expr(
                entity::Column::DeletedAt,
                Expr::value(chrono::Utc::now()),
            )
            .filter(entity::Column::Type.eq(setting_type))
            .filter(entity::Column::TenantId.eq(tenant_id))
            .filter(entity::Column::DomainObjectId.eq(domain_object_id))
            .exec(&*self.db)
            .await?;

        Ok(())
    }

    async fn hard_delete(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<()> {
        entity::Entity::delete_many()
            .filter(entity::Column::Type.eq(setting_type))
            .filter(entity::Column::TenantId.eq(tenant_id))
            .filter(entity::Column::DomainObjectId.eq(domain_object_id))
            .exec(&*self.db)
            .await?;

        Ok(())
    }

    async fn list_all(&self, limit: u64, offset: u64) -> Result<Vec<Setting>> {
        let results = entity::Entity::find()
            .filter(entity::Column::DeletedAt.is_null())
            .order_by_asc(entity::Column::Type)
            .order_by_asc(entity::Column::TenantId)
            .limit(limit)
            .offset(offset)
            .all(&*self.db)
            .await?;

        Ok(results.into_iter().map(|e| e.into()).collect())
    }
}

// ===== GTS Type Repository =====

pub struct SeaOrmGtsTypeRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmGtsTypeRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl GtsTypeRepository for SeaOrmGtsTypeRepository {
    async fn create(&self, gts_type: &GtsType) -> Result<GtsType> {
        let active_model: entity::gts_type::ActiveModel = gts_type.into();
        
        let result = entity::gts_type::Entity::insert(active_model)
            .exec_with_returning(&*self.db)
            .await?;

        result.try_into()
    }

    async fn find_by_type(&self, type_id: &str) -> Result<Option<GtsType>> {
        let result = entity::gts_type::Entity::find_by_id(type_id)
            .one(&*self.db)
            .await?;

        match result {
            Some(entity) => Ok(Some(entity.try_into()?)),
            None => Ok(None),
        }
    }

    async fn list_all(&self) -> Result<Vec<GtsType>> {
        let results = entity::gts_type::Entity::find()
            .order_by_asc(entity::gts_type::Column::Type)
            .all(&*self.db)
            .await?;

        results
            .into_iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<_>>>()
    }

    async fn update(&self, gts_type: &GtsType) -> Result<GtsType> {
        let active_model: entity::gts_type::ActiveModel = gts_type.into();
        
        let result = entity::gts_type::Entity::update(active_model)
            .exec(&*self.db)
            .await?;

        result.try_into()
    }

    async fn delete(&self, type_id: &str) -> Result<()> {
        entity::gts_type::Entity::delete_by_id(type_id)
            .exec(&*self.db)
            .await?;

        Ok(())
    }

    async fn exists(&self, type_id: &str) -> Result<bool> {
        let count = entity::gts_type::Entity::find_by_id(type_id)
            .count(&*self.db)
            .await?;

        Ok(count > 0)
    }
}
