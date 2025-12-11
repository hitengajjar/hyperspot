//! ModKit module declaration and lifecycle implementation

use crate::config::Config;
use crate::domain::Service;
use anyhow::Result;
use modkit::context::ModuleCtx;
use parking_lot::RwLock;
use std::sync::Arc;

/// Settings service module
#[modkit::module(
    name = "settings_service",
    deps = ["db"],
    capabilities = [db, rest],
    client = crate::contract::client::SettingsApi,
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct SettingsServiceModule {
    config: RwLock<Config>,
    service: RwLock<Option<Arc<Service>>>,
}

impl Default for SettingsServiceModule {
    fn default() -> Self {
        Self {
            config: RwLock::new(Config::default()),
            service: RwLock::new(None),
        }
    }
}

#[async_trait::async_trait]
impl modkit::Module for SettingsServiceModule {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // Read typed configuration
        let cfg = ctx.config::<Config>()?;
        *self.config.write() = cfg;

        // Get database handle
        let db = ctx.db_required()?;
        
        // Use secure ORM layer - SecureConn wraps DatabaseConnection
        let secure_conn = db.sea_secure();
        let conn = Arc::new(secure_conn.conn().clone());

        // Build repositories
        let settings_repo = Arc::new(crate::infra::storage::repositories::SeaOrmSettingsRepository::new(conn.clone()));
        let gts_type_repo = Arc::new(crate::infra::storage::repositories::SeaOrmGtsTypeRepository::new(conn));

        // Build event publisher (NoOp for now, can be replaced with real implementation)
        let event_publisher = Arc::new(crate::domain::NoOpEventPublisher);

        // Build domain service
        let service = Arc::new(Service::new(settings_repo, gts_type_repo, event_publisher));
        *self.service.write() = Some(service.clone());

        // Register native client in ClientHub for in-process calls
        let client = Arc::new(crate::api::native::NativeClient::new(service.clone()));
        ctx.client_hub().register::<dyn crate::contract::SettingsApi>(client);

        tracing::info!("Settings service initialized with native client registered");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Serve entry point for lifecycle
impl SettingsServiceModule {
    pub async fn serve(
        self: Arc<Self>,
        _cancel: tokio_util::sync::CancellationToken,
        _ready: modkit::lifecycle::ReadySignal,
    ) -> Result<()> {
        // Module is ready - lifecycle managed by ModKit
        // Ready signal is handled automatically by ModKit
        Ok(())
    }
}

// DbModule implementation for database migrations
#[async_trait::async_trait]
impl modkit::contracts::DbModule for SettingsServiceModule {
    async fn migrate(&self, db: &modkit_db::DbHandle) -> Result<()> {
        use crate::infra::storage::migrations::Migrator;
        use sea_orm_migration::MigratorTrait;
        
        let secure_conn = db.sea_secure();
        Migrator::up(secure_conn.conn(), None).await?;
        tracing::info!("Settings service migrations completed");
        Ok(())
    }
}

// RestfulModule implementation for REST API registration
impl modkit::contracts::RestfulModule for SettingsServiceModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn modkit::api::OpenApiRegistry,
    ) -> Result<axum::Router> {
        let service = self
            .service
            .read()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();
        
        tracing::info!("Registering settings service REST routes");
        crate::api::rest::routes::register_routes(router, openapi, service)
    }
}
