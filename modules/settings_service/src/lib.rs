//! Settings Service Module
//!
//! GTS-based configuration management system for HyperSpot.
//! Settings are identified by Global Type System (GTS) identifiers with
//! configuration metadata defined as GTS traits.

// Public exports
pub mod contract;
pub use contract::{
    client::SettingsApi, error::SettingsError, GtsTraits, GtsType, DomainType, EventConfig,
    EventTarget, Setting, SettingOptions,
};

pub mod module;
pub use module::SettingsServiceModule;

// Internal modules (hidden from public API)
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
