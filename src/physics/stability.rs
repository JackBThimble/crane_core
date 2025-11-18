use nalgebra as na;
use crate::types::*;
use crate::physics::statics::*;

/// Stability analysis for a crane configuration
#[derive(Debug, Clone)]
pub struct StabilityAnalysis {
    /// Overturning moment (trying to tip the crane)
    pub overturning_moment: f64,
    
    /// Restoring moment (keeping the crane stable)
    pub restoring_moment: f64,
    
    /// Stability factor (restoring / overturning)
    /// Must be > 1.0, OSHA requires > 1.5 typically
    pub stability_factor: f64,
    
    /// Tipping edge/axis
    pub tipping_edge: TippingEdge,
}

#[derive(Debug, Clone, Copy)]
pub enum TippingEdge {
    /// Tipping over front edge
    Front,
    /// Tipping over rear edge
    Rear,
    /// Tipping over left side
    Left,
    /// Tipping over right side
    Right,
}

impl TippingEdge {
    /// Get the tipping axis as a unit vector
    pub fn axis(&self) -> na::Unit<na::Vector3<f64>> {
        match self {
            TippingEdge::Front => na::Unit::new_normalize(na::Vector3::x()), // Tips about X axis
            TippingEdge::Rear => na::Unit::new_normalize(na::Vector3::x()),
            TippingEdge::Left => na::Unit::new_normalize(na::Vector3::z()),  // Tips about Z axis
            TippingEdge::Right => na::Unit::new_normalize(na::Vector3::z()),
        }
    }
}

/// Calculate stability for a mobile crane
/// 
/// This is the critical calculation that determines if your crane eats shit.
pub fn calculate_stability(
    crane_cog: na::Point3<f64>,
    crane_weight: Mass,
    load_position: na::Point3<f64>,
    load_weight: Mass,
    tipping_edge: na::Point3<f64>,
    tipping_axis: na::Unit<na::Vector3<f64>>,
) -> StabilityAnalysis {
    // Create force vectors for crane and load
    let crane_force = ForceVector::from_weight(crane_weight, crane_cog);
    let load_force = ForceVector::from_weight(load_weight, load_position);
    
    // Calculate moments about tipping edge
    let crane_moment = moment_about_axis(&crane_force, tipping_edge, tipping_axis);
    let load_moment = moment_about_axis(&load_force, tipping_edge, tipping_axis);
    
    // Restoring moment keeps crane stable (negative moment pulls away from tip)
    // Overturning moment tries to tip crane (positive moment)
    let restoring_moment = crane_moment.abs();
    let overturning_moment = load_moment.abs();
    
    let stability_factor = if overturning_moment > 0.0 {
        restoring_moment / overturning_moment
    } else {
        f64::INFINITY // No overturning force
    };
    
    StabilityAnalysis {
        overturning_moment,
        restoring_moment,
        stability_factor,
        tipping_edge: TippingEdge::Front, // TODO: determine which edge
    }
}

/// Outrigger configuration for mobile crane
#[derive(Debug, Clone)]
pub struct OutriggerConfig {
    /// Outrigger positions (feet, Y-up Z-forward)
    pub positions: Vec<na::Point3<f64>>,
    
    /// Maximum allowable reaction force per outrigger
    pub max_reactions: Vec<Force>,
}

impl OutriggerConfig {
    /// Standard 4-point outrigger setup (square pattern)
    pub fn square(spread: Length, max_load_per_pad: Force) -> Self {
        let half_spread = spread.get::<foot>() / 2.0;
        
        Self {
            positions: vec![
                na::Point3::new(half_spread, 0.0, half_spread),   // Front right
                na::Point3::new(-half_spread, 0.0, half_spread),  // Front left
                na::Point3::new(half_spread, 0.0, -half_spread),  // Rear right
                na::Point3::new(-half_spread, 0.0, -half_spread), // Rear left
            ],
            max_reactions: vec![max_load_per_pad; 4],
        }
    }
}

/// Calculate outrigger reaction forces for static equilibrium
/// 
/// Uses method of joints - solves for reaction forces at each support point
/// that satisfy ΣF = 0 and ΣM = 0
pub fn calculate_outrigger_reactions(
    config: &OutriggerConfig,
    system_cog: na::Point3<f64>,
    total_weight: Mass,
) -> Vec<Force> {
    let num_outriggers = config.positions.len();
    
    // For 4 outriggers (statically determinate):
    // We have 3 equilibrium equations (ΣFy=0, ΣMx=0, ΣMz=0) plus assumption of no uplift
    // This gives us 4 reactions
    
    if num_outriggers == 4 {
        // Use moment equilibrium about X and Z axes
        return solve_four_point_reactions(config, system_cog, total_weight);
    }
    
    // For other configurations, need more sophisticated solver
    // TODO: Implement least-squares or matrix inversion method
    vec![Force::new::<pound_force>(0.0); num_outriggers]
}

fn solve_four_point_reactions(
    config: &OutriggerConfig,
    cog: na::Point3<f64>,
    weight: Mass,
) -> Vec<Force> {
    let w = weight.get::<pound>();
    let positions = &config.positions;
    
    // Assuming symmetric layout (corners of a rectangle)
    // R1 + R2 + R3 + R4 = W (vertical equilibrium)
    // Moment about X axis: (R1+R2)*z_front - (R3+R4)*z_rear = W*cog.z
    // Moment about Z axis: (R1+R3)*x_right - (R2+R4)*x_left = W*cog.x
    
    // This is a simplified solution assuming rectangular outrigger pattern
    let x = cog.x;
    let z = cog.z;
    let x_spread = (positions[0].x - positions[1].x).abs();
    let z_spread = (positions[0].z - positions[2].z).abs();
    
    // Calculate reactions based on lever arms
    let r1 = w * 0.25 * (1.0 + 2.0*x/x_spread) * (1.0 + 2.0*z/z_spread);
    let r2 = w * 0.25 * (1.0 - 2.0*x/x_spread) * (1.0 + 2.0*z/z_spread);
    let r3 = w * 0.25 * (1.0 + 2.0*x/x_spread) * (1.0 - 2.0*z/z_spread);
    let r4 = w * 0.25 * (1.0 - 2.0*x/x_spread) * (1.0 - 2.0*z/z_spread);
    
    vec![
        Force::new::<pound_force>(r1.max(0.0)),
        Force::new::<pound_force>(r2.max(0.0)),
        Force::new::<pound_force>(r3.max(0.0)),
        Force::new::<pound_force>(r4.max(0.0)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_stability_calculation() {
        // Crane with COG at origin, 100k lbs
        let crane_cog = na::Point3::origin();
        let crane_weight = Mass::new::<pound>(100000.0);
        
        // Load at 50 ft radius, 10k lbs
        let load_pos = na::Point3::new(50.0, 10.0, 0.0);
        let load_weight = Mass::new::<pound>(10000.0);
        
        // Tipping edge at rear, 10 ft behind COG
        let tipping_edge = na::Point3::new(0.0, 0.0, -10.0);
        let tipping_axis = na::Unit::new_normalize(na::Vector3::x());
        
        let stability = calculate_stability(
            crane_cog,
            crane_weight,
            load_pos,
            load_weight,
            tipping_edge,
            tipping_axis,
        );
        
        // Crane weight * distance to edge = 100k * 10 = 1M ft-lb restoring
        // Load weight * distance to edge = 10k * 60 = 600k ft-lb overturning
        // Stability factor = 1M / 600k = 1.67 (safe!)
        
        println!("Restoring: {}, Overturning: {}, Factor: {}", 
            stability.restoring_moment,
            stability.overturning_moment,
            stability.stability_factor
        );
        
        assert!(stability.stability_factor > 1.5);
    }
    
    #[test]
    fn test_outrigger_reactions_centered_load() {
        let config = OutriggerConfig::square(
            Length::new::<foot>(20.0),
            Force::new::<pound_force>(50000.0),
        );
        
        // Load centered over crane base
        let cog = na::Point3::new(0.0, 5.0, 0.0);
        let weight = Mass::new::<pound>(40000.0);
        
        let reactions = calculate_outrigger_reactions(&config, cog, weight);
        
        // Each outrigger should carry 1/4 of the load
        for reaction in reactions {
            assert_relative_eq!(
                reaction.get::<pound_force>(),
                10000.0,
                epsilon = 100.0
            );
        }
    }
}
