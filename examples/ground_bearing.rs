use crane_core::physics::ground_bearing::*;
use crane_core::types::*;

fn main() {
    println!("=== Ground Bearing Pressure Analysis ===\n");

    let s = "-".repeat(50);

    println!("Test 1: Centered Load");
    println!("{}", s);
    test_centered_load();

    println!("\n");

    println!("Test 2: Side Load (Boom at 90 degrees)");
    println!("{}", s);
    test_side_load();

    println!("Test 3: Over-Rear Load");
    println!("{}", s);
    test_rear_load();

    println!("\n");

    println!("Test 4: Soil Capacity Validation");
    println!("{}", s);
    test_soil_validation();

    println!("\n");

    println!("Test 5: Required Mat Area");
    println!("{}", s);
    test_mat_sizing();

    println!("\n");

    println!("Test 6: Centered Load");
    println!("{}", s);
    test_centered_load_2();

    println!("\n");

    println!("Test 7: Side Load (Boom at 90 degrees)");
    println!("{}", s);
    test_side_load_2();

    println!("\n");

    println!("Test 8: Over-Rear Load");
    println!("{}", s);
    test_rear_load_2();

    println!("\n");

    println!("Test 9: Soil Capacity Validation");
    println!("{}", s);
    test_soil_validation_2();

    println!("\n");

    println!("Test 10: Required Mat Area");
    println!("{}", s);
    test_mat_sizing_2();

    println!("\n");

    println!("Test 11: Tipping Condition");
    println!("{}", s);
    test_tipping_condition();
}

fn test_centered_load() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(8.0), // 8ft high COG
            Length::new::<foot>(0.0),
        ),
        Mass::new::<pound>(50000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(60.0),
            Length::new::<foot>(0.0),
        ),
    );

    let pad_area = Area::new::<square_foot>(4.0);
    let spread = 10.0;

analysis.add_support("Front-Left", 
        Length::new::<foot>(-spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(spread), 
        pad_area);
    analysis.add_support("Front-Right", 
        Length::new::<foot>(spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(spread), 
        pad_area);
    analysis.add_support("Rear-Left", 
        Length::new::<foot>(-spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(-spread), 
        pad_area);
    analysis.add_support("Rear-Right", 
        Length::new::<foot>(spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(-spread), 
        pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("{}", result.summary());
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_side_load() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(8.0),
            Length::new::<foot>(0.0),
        ),
        Mass::new::<pound>(50000.0),
        (
            Length::new::<foot>(80.0),   // 80 ft to the right (side load)
            Length::new::<foot>(50.0),
            Length::new::<foot>(0.0),
        ),
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    let spread = 10.0;
    
    analysis.add_support("Front-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Front-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Rear-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    analysis.add_support("Rear-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("{}", result.summary());
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_rear_load() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(8.0),
            Length::new::<foot>(0.0),
        ),
        Mass::new::<pound>(50000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(40.0),
            Length::new::<foot>(-60.0),  // 60 ft behind crane (over-rear)
        ),
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    let spread = 10.0;
    
    analysis.add_support("Front-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Front-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Rear-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    analysis.add_support("Rear-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("{}", result.summary());
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_soil_validation() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(60.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)),
    );
    
    // Small pads
    let small_pad = Area::new::<square_foot>(1.0);
    analysis.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), small_pad);
    analysis.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), small_pad);
    analysis.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), small_pad);
    analysis.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), small_pad);
    
    println!("Testing on soft clay (1000 psf / 10 PSI):");
    match analysis.validate_soil_capacity(soil_capacities::soft_clay()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }
    
    println!("\nTesting on dense sand (5000 psf / 50 PSI):");
    match analysis.validate_soil_capacity(soil_capacities::dense_sand()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }
    
    // Now with larger pads
    let mut analysis2 = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(60.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)),
    );
    
    let large_pad = Area::new::<square_foot>(16.0);  // 4x4 ft mats
    analysis2.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), large_pad);
    analysis2.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), large_pad);
    analysis2.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), large_pad);
    analysis2.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), large_pad);
    
    println!("\nWith 4x4 ft mats on soft clay:");
    match analysis2.validate_soil_capacity(soil_capacities::soft_clay()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }
}

fn test_mat_sizing() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(80.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)),
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    analysis.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), pad_area);
    analysis.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), pad_area);
    analysis.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), pad_area);
    analysis.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), pad_area);
    
    println!("Current pad size: 2x2 ft (4 sq ft)");
    
    match analysis.required_mat_area(soil_capacities::soft_clay(), 2.0) {
        Ok(area) => {
            let sq_ft = area.get::<square_foot>();
            let side = sq_ft.sqrt();
            println!("\nFor soft clay with 2:1 safety factor:");
            println!("  Required area: {:.1} sq ft", sq_ft);
            println!("  Square mat size: {:.1} ft x {:.1} ft", side, side);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    match analysis.required_mat_area(soil_capacities::medium_clay(), 2.0) {
        Ok(area) => {
            let sq_ft = area.get::<square_foot>();
            let side = sq_ft.sqrt();
            println!("\nFor medium clay with 2:1 safety factor:");
            println!("  Required area: {:.1} sq ft", sq_ft);
            println!("  Square mat size: {:.1} ft x {:.1} ft", side, side);
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn test_centered_load_2() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),  // 100k lb crane
        (
            Length::new::<foot>(0.0),   // Centered
            Length::new::<foot>(8.0),   // 8 ft high COG
            Length::new::<foot>(0.0),
        ),
        Mass::new::<pound>(50000.0),   // 50k lb load
        (
            Length::new::<foot>(0.0),   // Centered
            Length::new::<foot>(60.0),  // 60 ft hook height
            Length::new::<foot>(0.0),
        ),
    );
    
    // 20 ft outrigger spread, 2x2 ft pads
    let pad_area = Area::new::<square_foot>(4.0);
    let spread = 10.0;
    
    analysis.add_support("Front-Left", 
        Length::new::<foot>(-spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(spread), 
        pad_area);
    analysis.add_support("Front-Right", 
        Length::new::<foot>(spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(spread), 
        pad_area);
    analysis.add_support("Rear-Left", 
        Length::new::<foot>(-spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(-spread), 
        pad_area);
    analysis.add_support("Rear-Right", 
        Length::new::<foot>(spread), 
        Length::new::<foot>(0.0), 
        Length::new::<foot>(-spread), 
        pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("{}", result.summary());
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_side_load_2() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(8.0),
            Length::new::<foot>(0.0),
        ),
        Mass::new::<pound>(50000.0),
        (
            Length::new::<foot>(30.0),   // 30 ft to the right (realistic)
            Length::new::<foot>(50.0),
            Length::new::<foot>(0.0),
        ),
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    let spread = 10.0;
    
    analysis.add_support("Front-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Front-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Rear-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    analysis.add_support("Rear-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("{}", result.summary());
            println!("\nNote: Right side outriggers carry more load due to side moment");
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_rear_load_2() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(8.0),
            Length::new::<foot>(0.0),
        ),
        Mass::new::<pound>(50000.0),
        (
            Length::new::<foot>(0.0),
            Length::new::<foot>(40.0),
            Length::new::<foot>(-25.0),  // 25 ft behind crane (realistic)
        ),
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    let spread = 10.0;
    
    analysis.add_support("Front-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Front-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(spread), pad_area);
    analysis.add_support("Rear-Left", Length::new::<foot>(-spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    analysis.add_support("Rear-Right", Length::new::<foot>(spread), Length::new::<foot>(0.0), Length::new::<foot>(-spread), pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("{}", result.summary());
            println!("\nNote: Rear outriggers carry more load due to over-rear moment");
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_soil_validation_2() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(30.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)),
    );
    
    // Small pads
    let small_pad = Area::new::<square_foot>(4.0);
    analysis.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), small_pad);
    analysis.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), small_pad);
    analysis.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), small_pad);
    analysis.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), small_pad);
    
    println!("With 2x2 pads (4 sq ft):\n");

    println!("Testing on soft clay (1000 psf / 10 PSI):");
    match analysis.validate_soil_capacity(soil_capacities::soft_clay()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    println!("\nTesting on medium clay (2500 psf / 25 PSI):");
    match analysis.validate_soil_capacity(soil_capacities::medium_clay()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }
    
    println!("\nTesting on dense sand (5000 psf / 50 PSI):");
    match analysis.validate_soil_capacity(soil_capacities::dense_sand()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    println!("\nTesting on gravel (8000 psf / 80 PSI):");
    match analysis.validate_soil_capacity(soil_capacities::gravel()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    
    // Now with larger pads
    println!("\n--- With 4x4 mats (16 sq ft) ---\n");

    let mut analysis2 = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(30.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)),
    );
    
    let large_pad = Area::new::<square_foot>(16.0);  // 4x4 ft mats
    analysis2.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), large_pad);
    analysis2.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), large_pad);
    analysis2.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), large_pad);
    analysis2.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), large_pad);
    
    println!("\nTesting on soft clay (1000 psf / 10 PSI):");
    match analysis2.validate_soil_capacity(soil_capacities::soft_clay()) {
        Ok(_) => println!("  ✅ PASS - Soil capacity adequate"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    println!("\nTesting on medium clay (2500 psf / 25PSI):");
    match analysis2.validate_soil_capacity(soil_capacities::medium_clay()) {
        Ok(_) => println!(" ✅ PASS - Soil capacity adequate with mats"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }
}

fn test_mat_sizing_2() {
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(30.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)),
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    analysis.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), pad_area);
    analysis.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), pad_area);
    analysis.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), pad_area);
    analysis.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), pad_area);
    
    println!("Current pad size: 2x2 ft (4 sq ft)");
    
    match analysis.required_mat_area(soil_capacities::soft_clay(), 2.0) {
        Ok(area) => {
            let sq_ft = area.get::<square_foot>();
            let side = sq_ft.sqrt();
            println!("\nFor soft clay with 2:1 safety factor:");
            println!("  Required area: {:.1} sq ft", sq_ft);
            println!("  Square mat size: {:.1} ft x {:.1} ft", side, side);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    match analysis.required_mat_area(soil_capacities::medium_clay(), 2.0) {
        Ok(area) => {
            let sq_ft = area.get::<square_foot>();
            let side = sq_ft.sqrt();
            println!("\nFor medium clay with 2:1 safety factor:");
            println!("  Required area: {:.1} sq ft", sq_ft);
            println!("  Square mat size: {:.1} ft x {:.1} ft", side, side);
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn test_tipping_condition() {
    println!("Testing extreme side load that SHOULD fail...\n");
    
    let mut analysis = GroundBearingAnalysis::new(
        Mass::new::<pound>(100000.0),
        (Length::new::<foot>(0.0), Length::new::<foot>(8.0), Length::new::<foot>(0.0)),
        Mass::new::<pound>(50000.0),
        (Length::new::<foot>(80.0), Length::new::<foot>(50.0), Length::new::<foot>(0.0)), // EXTREME
    );
    
    let pad_area = Area::new::<square_foot>(4.0);
    analysis.add_support("FL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), pad_area);
    analysis.add_support("FR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(10.0), pad_area);
    analysis.add_support("RL", Length::new::<foot>(-10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), pad_area);
    analysis.add_support("RR", Length::new::<foot>(10.0), Length::new::<foot>(0.0), Length::new::<foot>(-10.0), pad_area);
    
    match analysis.calculate_reactions() {
        Ok(result) => {
            println!("❌ UNEXPECTED: Crane stable at 80 ft radius!");
            println!("{}", result.summary());
        }
        Err(e) => {
            println!("✅ CORRECT: Tipping detected as expected");
            println!("   {}", e);
            println!("\n   This is a safety feature - the crane would tip over in this configuration!");
        }
    }
}
