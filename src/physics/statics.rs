use nalgebra as na;
use crate::types::*;
use crate::types::units::*;

/// A force vector in 3D space with magnitude and direction
#[derive(Debug, Clone, Copy)]
pub struct ForceVector {
    /// Point of application (Y-up, Z-forward)
    pub point: na::Point3<f64>,
    /// Force vector (N or lbf, depending on your religion)
    pub force: na::Vector3<f64>,
}

impl ForceVector {
    /// Create a new force vector
    /// point: location in feet, force: magnitude in pounds-force
    pub fn new(point: na::Point3<f64>, force: na::Vector3<f64>) -> Self {
        Self { point, force }
    }
    
    /// Gravity force for a mass at a point
    pub fn from_weight(weight: Weight, point: na::Point3<f64>) -> Self {
        let force_lbf = weight.get::<pound>(); // Weight in lbf = mass in lb on Earth
        Self {
            point,
            force: na::Vector3::new(0.0, -force_lbf, 0.0), // Down is -Y
        }
    }
    
    /// Magnitude of the force
    pub fn magnitude(&self) -> f64 {
        self.force.magnitude()
    }
}

/// Calculate moment (torque) about a point
/// 
/// Moment = r × F (cross product)
/// Returns moment vector (right-hand rule)
pub fn moment_about_point(
    force: &ForceVector,
    pivot: na::Point3<f64>,
) -> na::Vector3<f64> {
    let r = force.point - pivot; // Position vector from pivot to force application
    r.cross(&force.force)
}

/// Calculate scalar moment about an axis
/// 
/// For tipping calculations, we care about moment magnitude around a specific axis
pub fn moment_about_axis(
    force: &ForceVector,
    pivot: na::Point3<f64>,
    axis: na::Unit<na::Vector3<f64>>,
) -> f64 {
    let moment = moment_about_point(force, pivot);
    moment.dot(&axis.into_inner())
}

/// Sum all forces in a system
pub fn sum_forces(forces: &[ForceVector]) -> na::Vector3<f64> {
    forces.iter().map(|f| f.force).sum()
}

/// Sum all moments about a point
pub fn sum_moments(forces: &[ForceVector], pivot: na::Point3<f64>) -> na::Vector3<f64> {
    forces.iter()
        .map(|f| moment_about_point(f, pivot))
        .sum()
}

/// Check if a system is in static equilibrium
/// 
/// For equilibrium:
/// - Sum of forces = 0
/// - Sum of moments = 0
pub fn is_in_equilibrium(
    forces: &[ForceVector],
    pivot: na::Point3<f64>,
    force_tolerance: f64,
    moment_tolerance: f64,
) -> bool {
    let net_force = sum_forces(forces);
    let net_moment = sum_moments(forces, pivot);
    
    net_force.magnitude() < force_tolerance && 
    net_moment.magnitude() < moment_tolerance
}

/// Calculate center of gravity for multiple point masses
pub fn center_of_gravity(masses: &[(Weight, na::Point3<f64>)]) -> na::Point3<f64> {
    let total_weight: f64 = masses.iter()
        .map(|(w, _)| w.get::<pound>())
        .sum();
    
    if total_weight == 0.0 {
        return na::Point3::origin();
    }
    
    let weighted_sum = masses.iter()
        .map(|(w, p)| p.coords * w.get::<pound>())
        .sum::<na::Vector3<f64>>();
    
    na::Point3::from(weighted_sum / total_weight)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_moment_calculation() {
        // 1000 lb load at 50 ft radius should produce 50,000 ft-lb moment
        let force = ForceVector::from_weight(
            Weight::new::<pound>(1000.0),
            na::Point3::new(50.0, 0.0, 0.0), // 50 ft to the side
        );
        
        let pivot = na::Point3::origin();
        let moment = moment_about_point(&force, pivot);
        
        // Moment should be in Z direction (right-hand rule: X × -Y = Z)
        assert_relative_eq!(moment.z.abs(), 50000.0, epsilon = 0.1);
    }
    
    #[test]
    fn test_center_of_gravity() {
        let masses = vec![
            (Weight::new::<pound>(1000.0), na::Point3::new(0.0, 0.0, 0.0)),
            (Weight::new::<pound>(1000.0), na::Point3::new(10.0, 0.0, 0.0)),
        ];
        
        let cog = center_of_gravity(&masses);
        
        // COG should be at midpoint
        assert_relative_eq!(cog.x, 5.0, epsilon = 0.001);
        assert_relative_eq!(cog.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(cog.z, 0.0, epsilon = 0.001);
    }
    
    #[test]
    fn test_equilibrium() {
        // Balanced see-saw: equal weights at equal distances
        let forces = vec![
            ForceVector::from_weight(
                Weight::new::<pound>(1000.0),
                na::Point3::new(-10.0, 0.0, 0.0),
            ),
            ForceVector::from_weight(
                Weight::new::<pound>(1000.0),
                na::Point3::new(10.0, 0.0, 0.0),
            ),
            // Reaction force at pivot
            ForceVector::new(
                na::Point3::origin(),
                na::Vector3::new(0.0, 2000.0, 0.0),
            ),
        ];
        
        assert!(is_in_equilibrium(&forces, na::Point3::origin(), 0.1, 0.1));
    }
}