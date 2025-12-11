//! SeaORM entities for database tables

use sea_orm::entity::prelude::*;

/// Settings table entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    /// GTS identifier (part of composite primary key)
    #[sea_orm(primary_key, auto_increment = false)]
    pub r#type: String,
    
    /// Tenant ID (part of composite primary key)
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: Uuid,
    
    /// Domain object ID (part of composite primary key)
    #[sea_orm(primary_key, auto_increment = false)]
    pub domain_object_id: String,
    
    /// Setting value as JSON
    pub data: Json,
    
    /// Creation timestamp
    pub created_at: DateTimeUtc,
    
    /// Last update timestamp
    pub updated_at: DateTimeUtc,
    
    /// Soft delete timestamp
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Foreign key to cti_registry
    #[sea_orm(
        belongs_to = "gts_type::Entity",
        from = "Column::Type",
        to = "gts_type::Column::Type"
    )]
    GtsType,
}

impl Related<gts_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GtsType.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// GTS type registry module
pub mod gts_type {
    use sea_orm::entity::prelude::*;

    /// GTS type registry table entity
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "cti_registry")]
    pub struct Model {
        /// GTS identifier (primary key)
        #[sea_orm(primary_key, auto_increment = false)]
        pub r#type: String,
        
        /// GTS traits as JSON (domain_type, events, options, operations)
        pub traits: Json,
        
        /// JSON Schema for validation (optional)
        pub schema: Option<Json>,
        
        /// Creation timestamp
        pub created_at: DateTimeUtc,
        
        /// Last update timestamp
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        /// One-to-many relationship with settings
        #[sea_orm(has_many = "super::Entity")]
        Settings,
    }

    impl Related<super::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Settings.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
