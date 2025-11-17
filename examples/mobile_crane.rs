use crane_core::equipment::{Crane, MobileCrane};
use crane_core::capacity::LoadChartPackage;
use crane_core::types::*;
use crane_core::types::units::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create mobile crane
    let mut crane = MobileCrane::new(
        "Grove",
        "GMK5250L",
        Distance::new::<foot>(154.2),
        Distance::new::<foot>(10.0),
    );
    
    // Load charts from file
    crane.load_charts_from_file("crane_core/src/data/cranes/grove_gmk5250l/grove_gmk5250l_data.json")?;
    
    println!("Loaded crane: {} {}", crane.manufacturer, crane.model);
    
    // Set boom angle
    crane.boom_angle = Angle::new::<degree>(60.0);
    
    // Get current configuration
    let config = crane.configuration();
    println!("Current radius: {} ft", config.radius.get::<foot>());
    
    // Get rated capacity
    let capacity = crane.rated_capacity();
    println!("Rated capacity at current config: {} lbs", capacity.get::<pound>());
    
    // Validate a lift
    let load = Weight::new::<pound>(50000.0);
    match crane.validate_lift(load) {
        Ok(_) => println!("Lift is safe!"),
        Err(e) => println!("Lift failed: {}", e),
    }
    
    Ok(())
}
