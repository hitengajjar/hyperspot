//! Domain layer - business logic and services

pub mod events;
pub mod repository;
pub mod service;
pub mod validation;

pub use events::{EventPublisher, NoOpEventPublisher, SettingEvent};
pub use repository::{GtsTypeRepository, SettingsRepository};
pub use service::Service;
