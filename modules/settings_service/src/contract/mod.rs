//! Contract layer - public API for inter-module communication
//!
//! This layer contains transport-agnostic models and the native client trait.
//! NO serde derives on models - these are pure domain types.

pub mod client;
pub mod error;
pub mod model;

pub use client::SettingsApi;
pub use error::SettingsError;
pub use model::{
    GtsTraits, GtsType, DomainType, EventConfig, EventTarget, Setting, SettingOptions,
};
