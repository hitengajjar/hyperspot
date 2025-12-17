//! Tenant hierarchy client abstraction for account server integration

use async_trait::async_trait;
use uuid::Uuid;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// Error type for hierarchy service operations
#[derive(Debug, thiserror::Error)]
pub enum HierarchyError {
    #[error("Tenant not found: {0}")]
    TenantNotFound(Uuid),
    
    #[error("Hierarchy service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Invalid hierarchy structure: {0}")]
    InvalidHierarchy(String),
}

/// Trait for communicating with tenant hierarchy service
/// 
/// This abstraction allows switching between mock and real implementations
/// for testing and production use.
#[async_trait]
pub trait TenantHierarchyClient: Send + Sync {
    /// Get the parent tenant ID for a given tenant
    /// 
    /// Returns None if the tenant is a root tenant (no parent)
    /// Returns error if tenant does not exist
    async fn get_parent_tenant(&self, tenant_id: Uuid) -> Result<Option<Uuid>, HierarchyError>;
    
    /// Validate that a tenant exists in the hierarchy
    /// 
    /// Returns true if tenant exists, false otherwise
    async fn validate_tenant_exists(&self, tenant_id: Uuid) -> Result<bool, HierarchyError>;
    
    /// Get the full path from tenant to root
    /// 
    /// Returns ordered list from child to root (inclusive)
    /// First element is the requested tenant, last is root
    async fn get_tenant_path(&self, tenant_id: Uuid) -> Result<Vec<Uuid>, HierarchyError>;
}

#[derive(Clone, Default)]
pub struct NoOpTenantHierarchyClient;

#[async_trait]
impl TenantHierarchyClient for NoOpTenantHierarchyClient {
    async fn get_parent_tenant(&self, _tenant_id: Uuid) -> Result<Option<Uuid>, HierarchyError> {
        Ok(None)
    }

    async fn validate_tenant_exists(&self, _tenant_id: Uuid) -> Result<bool, HierarchyError> {
        Ok(true)
    }

    async fn get_tenant_path(&self, tenant_id: Uuid) -> Result<Vec<Uuid>, HierarchyError> {
        Ok(vec![tenant_id])
    }
}

/// Mock implementation of TenantHierarchyClient for testing
/// 
/// Stores hierarchy in memory as a parent map
#[derive(Clone)]
pub struct MockTenantHierarchyClient {
    /// Map of tenant_id -> parent_id
    /// None value indicates root tenant
    hierarchy: Arc<RwLock<HashMap<Uuid, Option<Uuid>>>>,
}

impl MockTenantHierarchyClient {
    /// Create a new mock hierarchy client
    pub fn new() -> Self {
        Self {
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a tenant to the mock hierarchy
    /// 
    /// # Arguments
    /// * `tenant_id` - The tenant to add
    /// * `parent_id` - The parent tenant (None for root)
    pub fn add_tenant(&self, tenant_id: Uuid, parent_id: Option<Uuid>) {
        self.hierarchy.write().insert(tenant_id, parent_id);
    }
    
    /// Remove a tenant from the mock hierarchy
    pub fn remove_tenant(&self, tenant_id: Uuid) {
        self.hierarchy.write().remove(&tenant_id);
    }
    
    /// Clear all tenants from the mock hierarchy
    pub fn clear(&self) {
        self.hierarchy.write().clear();
    }
    
    /// Get the number of tenants in the hierarchy
    pub fn tenant_count(&self) -> usize {
        self.hierarchy.read().len()
    }
}

impl Default for MockTenantHierarchyClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantHierarchyClient for MockTenantHierarchyClient {
    async fn get_parent_tenant(&self, tenant_id: Uuid) -> Result<Option<Uuid>, HierarchyError> {
        self.hierarchy
            .read()
            .get(&tenant_id)
            .copied()
            .ok_or(HierarchyError::TenantNotFound(tenant_id))
    }
    
    async fn validate_tenant_exists(&self, tenant_id: Uuid) -> Result<bool, HierarchyError> {
        Ok(self.hierarchy.read().contains_key(&tenant_id))
    }
    
    async fn get_tenant_path(&self, tenant_id: Uuid) -> Result<Vec<Uuid>, HierarchyError> {
        let hierarchy = self.hierarchy.read();
        
        // Check if tenant exists
        if !hierarchy.contains_key(&tenant_id) {
            return Err(HierarchyError::TenantNotFound(tenant_id));
        }
        
        let mut path = vec![tenant_id];
        let mut current = tenant_id;
        let mut visited = std::collections::HashSet::new();
        visited.insert(current);
        
        // Traverse up to root
        while let Some(Some(parent_id)) = hierarchy.get(&current) {
            // Detect circular references
            if visited.contains(parent_id) {
                return Err(HierarchyError::InvalidHierarchy(
                    format!("Circular reference detected at tenant {}", parent_id)
                ));
            }
            
            path.push(*parent_id);
            visited.insert(*parent_id);
            current = *parent_id;
            
            // Safety limit to prevent infinite loops
            if path.len() > 100 {
                return Err(HierarchyError::InvalidHierarchy(
                    "Hierarchy depth exceeds maximum (100 levels)".to_string()
                ));
            }
        }
        
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_client_add_tenant() {
        let client = MockTenantHierarchyClient::new();
        let tenant_id = Uuid::new_v4();
        
        client.add_tenant(tenant_id, None);
        
        assert_eq!(client.tenant_count(), 1);
        assert!(client.validate_tenant_exists(tenant_id).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_mock_client_get_parent() {
        let client = MockTenantHierarchyClient::new();
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        
        client.add_tenant(root_id, None);
        client.add_tenant(child_id, Some(root_id));
        
        let parent = client.get_parent_tenant(child_id).await.unwrap();
        assert_eq!(parent, Some(root_id));
        
        let root_parent = client.get_parent_tenant(root_id).await.unwrap();
        assert_eq!(root_parent, None);
    }
    
    #[tokio::test]
    async fn test_mock_client_get_tenant_path() {
        let client = MockTenantHierarchyClient::new();
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let grandchild_id = Uuid::new_v4();
        
        client.add_tenant(root_id, None);
        client.add_tenant(child_id, Some(root_id));
        client.add_tenant(grandchild_id, Some(child_id));
        
        let path = client.get_tenant_path(grandchild_id).await.unwrap();
        assert_eq!(path, vec![grandchild_id, child_id, root_id]);
    }
    
    #[tokio::test]
    async fn test_mock_client_tenant_not_found() {
        let client = MockTenantHierarchyClient::new();
        let non_existent = Uuid::new_v4();
        
        let result = client.get_parent_tenant(non_existent).await;
        assert!(matches!(result, Err(HierarchyError::TenantNotFound(_))));
    }
    
    #[tokio::test]
    async fn test_mock_client_circular_reference_detection() {
        let client = MockTenantHierarchyClient::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        
        // Create circular reference: A -> B -> A
        client.add_tenant(tenant_a, Some(tenant_b));
        client.add_tenant(tenant_b, Some(tenant_a));
        
        let result = client.get_tenant_path(tenant_a).await;
        assert!(matches!(result, Err(HierarchyError::InvalidHierarchy(_))));
    }
    
    #[tokio::test]
    async fn test_mock_client_clear() {
        let client = MockTenantHierarchyClient::new();
        client.add_tenant(Uuid::new_v4(), None);
        client.add_tenant(Uuid::new_v4(), None);
        
        assert_eq!(client.tenant_count(), 2);
        
        client.clear();
        assert_eq!(client.tenant_count(), 0);
    }
}
