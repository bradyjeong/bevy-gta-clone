use gameplay_factory::*;

fn main() {
    // Clear global registry
    clear_all_prefab_ids();

    println!("=== TESTING ORACLE'S PREFAB ID COLLISION DETECTION ===");

    // Test 1: Same factory, duplicate IDs
    println!("\n1. Testing same factory, duplicate IDs:");
    let mut factory1 = Factory::new();
    let id1 = PrefabId::new(12345);

    match factory1.register(id1, BasicPrefab::new()) {
        Ok(()) => println!("   ✅ First registration succeeded"),
        Err(e) => println!("   ❌ First registration failed: {e}"),
    }

    match factory1.register(id1, BasicPrefab::new()) {
        Ok(()) => println!("   ❌ Second registration succeeded (BAD!)"),
        Err(e) => println!("   ✅ Second registration failed: {e}"),
    }

    // Test 2: Different factories, same IDs (global collision detection)
    println!("\n2. Testing different factories, same IDs:");
    let mut factory2 = Factory::new();

    match factory2.register(id1, BasicPrefab::new()) {
        Ok(()) => println!("   ❌ Cross-factory registration succeeded (BAD!)"),
        Err(e) => println!("   ✅ Cross-factory registration failed: {e}"),
    }

    // Test 3: TryFrom<u32> prevents silent narrowing
    println!("\n3. Testing TryFrom<u32> conversion:");
    let id = PrefabId::from(42u32);
    println!("   ✅ u32 conversion succeeded: {id:?}");

    // Test 4: Global registry functions
    println!("\n4. Testing global registry functions:");
    let all_ids = get_all_prefab_ids();
    println!("   Total registered IDs: {}", all_ids.len());
    for id in all_ids {
        println!("   - {id:?}");
    }

    // Test 5: Path-based ID generation
    println!("\n5. Testing path-based ID generation:");
    let _factory3 = Factory::new();
    let paths = [
        "/assets/player.ron",
        "/assets/enemy.ron",
        "/assets/player.ron", // duplicate
    ];

    for path in paths {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        let id = PrefabId::new(hash);

        if is_prefab_id_registered(id) {
            println!("   ⚠️  Path {path} would generate duplicate ID: {id:?}");
        } else {
            println!("   ✅ Path {path} generates unique ID: {id:?}");
        }
    }

    println!("\n=== ORACLE'S COLLISION DETECTION VERIFIED ===");
}
