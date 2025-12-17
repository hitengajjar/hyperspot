//! Common test utilities and shared tenant hierarchy

use uuid::Uuid;

/// Realistic tenant hierarchy for testing
/// Root → Partners (Pax8, Datto, ConnectWise) → Customers
#[derive(Debug, Clone)]
pub struct TestTenantHierarchy {
    pub root: Uuid,
    pub partner1_pax8: Uuid,
    pub partner1_customer1_evergreen: Uuid,
    pub partner1_customer2_braden: Uuid,
    pub partner2_datto: Uuid,
    pub partner2_customer1_bcs: Uuid,
    pub partner2_customer2_computergeeks: Uuid,
    pub partner3_connectwise: Uuid,
    pub partner3_customer1_sanit: Uuid,
    pub partner3_customer2_sourcepass: Uuid,
}

impl TestTenantHierarchy {
    /// Create a new tenant hierarchy with fresh UUIDs
    pub fn new() -> Self {
        Self {
            root: Uuid::new_v4(),
            partner1_pax8: Uuid::new_v4(),
            partner1_customer1_evergreen: Uuid::new_v4(),
            partner1_customer2_braden: Uuid::new_v4(),
            partner2_datto: Uuid::new_v4(),
            partner2_customer1_bcs: Uuid::new_v4(),
            partner2_customer2_computergeeks: Uuid::new_v4(),
            partner3_connectwise: Uuid::new_v4(),
            partner3_customer1_sanit: Uuid::new_v4(),
            partner3_customer2_sourcepass: Uuid::new_v4(),
        }
    }
    
    /// Print the hierarchy structure
    pub fn print_structure(&self) {
        println!("\n📊 Tenant Hierarchy Structure:");
        println!("   Root: {}", self.root);
        println!("   ├─ Partner1 (Pax8): {}", self.partner1_pax8);
        println!("   │  ├─ Evergreen and Lyra: {}", self.partner1_customer1_evergreen);
        println!("   │  └─ Braden Business Systems: {}", self.partner1_customer2_braden);
        println!("   ├─ Partner2 (Datto): {}", self.partner2_datto);
        println!("   │  ├─ BCS Manufacturing: {}", self.partner2_customer1_bcs);
        println!("   │  └─ Computer Geeks: {}", self.partner2_customer2_computergeeks);
        println!("   └─ Partner3 (ConnectWise): {}", self.partner3_connectwise);
        println!("      ├─ San-iT: {}", self.partner3_customer1_sanit);
        println!("      └─ Sourcepass: {}", self.partner3_customer2_sourcepass);
    }
}

impl Default for TestTenantHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
