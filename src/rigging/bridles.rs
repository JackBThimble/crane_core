extern crate uom;

use crate::physics::statics::*;
use crate::rigging::{LegType, LiveLeg, Sling};
use crate::types::units::*;
use crate::types::*;
use nalgebra as na;
use uom::si::f64;
use uom::si::*;


/// A multi-leg bridle configuration
///
/// Can be all dead legs, all live legs, or mixed
#[derive(Debug, Clone)]
pub struct Bridle {
    /// Load being lifted
    pub load: Weight,

    /// Load center of gravity
    pub load_cog: na::Point3<f64>,

    /// Dead legs (slings)
    pub dead_legs: Vec<BridleLeg>,

    /// Live legs (chain falls, etc.)
    pub live_legs: Vec<LiveLeg>,

    /// Hook position (where all legs meet)
    pub hook_position: na::Point3<f64>,
}

#[derive(Debug, Clone)]
pub struct BridleLeg {
    /// The sling
    pub sling: Sling,

    /// Attachment point on load (relative to load COG)
    pub attachment_point: na::Point3<f64>,

    /// Calculated tension in this leg
    pub tension: Force,
}

impl Bridle {
    pub fn new(load: Weight, load_cog: na::Point3<f64>, hook_position: na::Point3<f64>) -> Self {
        Self {
            load,
            load_cog,
            dead_legs: Vec::new(),
            live_legs: Vec::new(),
            hook_position,
        }
    }

    /// Add a dead leg (sling) to the bridle
    pub fn add_dead_leg(&mut self, sling: Sling, attachment_point: na::Point3<f64>) {
        self.dead_legs.push(BridleLeg {
            sling,
            attachment_point,
            tension: Force::new::<pound_force>(0.0),
        });
    }

    /// Add a live leg to the bridle
    pub fn add_live_leg(&mut self, live_leg: LiveLeg) {
        self.live_legs.push(live_leg);
    }

    /// Calculate load distribution in the bridle
    ///
    /// For all dead legs: solve using geometry (static equilibrium)
    /// For mixed: live legs set their tension, dead legs take remainder
    /// For all live legs: need to specify tensions manually
    pub fn calculate_load_distribution(&mut self) -> Result<BridleAnalysis, BridleError> {
        let total_dead_legs = self.dead_legs.len();
        let total_live_legs = self.live_legs.len();

        if total_dead_legs == 0 && total_live_legs == 0 {
            return Err(BridleError::NoLegs);
        }

        // Case 1: All dead legs - pure geometry-based distribution
        if total_live_legs == 0 {
            return self.calculate_dead_leg_distribution();
        }

        // Case 2: All live legs - verify tension sums to load
        if total_dead_legs == 0 {
            return self.verify_live_leg_distribution();
        }

        // Case 3: Mixed - live legs set, dead legs react
        self.calculate_mixed_distribution()
    }

    /// Calculate distribution for all dead legs
    ///
    /// Uses geometry and assumes equal load sharing if symmetric
    /// For asymmetric, uses moment equilibrium
    fn calculate_dead_leg_distribution(&mut self) -> Result<BridleAnalysis, BridleError> {
        let num_legs = self.dead_legs.len();

        if num_legs == 0 {
            return Err(BridleError::NoLegs);
        }

        // For now, implement simplified approach for symmetric bridles
        // TODO: Implement full moment equilibrium solver for asymmetric cases

        if self.is_symmetric() {
            // Symmetric case: equal load sharing
            let load_per_leg = self.load.get::<pound>() / num_legs as f64;

            for leg in &mut self.dead_legs {
                // Calculate angle from vertical
                let attachment_world = self.load_cog + leg.attachment_point.coords;
                let leg_vector = self.hook_position - attachment_world;
                let leg_length = leg_vector.magnitude();
                let vertical_component = leg_vector.y;

                let angle_from_vertical =
                                    Angle::new::<radian>((vertical_component / leg_length).acos());
                    
                // Tension = load_per_leg / cos(angle)
                let tension = load_per_leg / angle_from_vertical.get::<radian>().cos();
                leg.tension = Force::new::<pound_force>(tension);

                // Update sling configuration with calculated angle
                // (This is for capacity checking)
            }
        } else {
            // Asymmetric case: use moment equilibrium
            return self.calculate_asymmetric_distribution();
        }

        // Verify all legs are within capacity
        for leg in &self.dead_legs {
            if !leg.sling.is_safe(leg.tension) {
                return Err(BridleError::LegOverCapacity {
                    leg_id: leg.sling.id.clone(),
                    tension: DisplayForce(leg.tension),
                    capacity: DisplayWeight(leg.sling.effective_capacity()),
                });
            }
        }

        Ok(BridleAnalysis {
            total_load: self.load,
            dead_leg_tensions: self.dead_legs.iter().map(|l| l.tension).collect(),
            live_leg_tensions: Vec::new(),
            is_balanced: true,
        })
    }

    /// Check if bridle is symmetric (equal leg lengths and angles)
    fn is_symmetric(&self) -> bool {
        if self.dead_legs.len() < 2 {
            return true;
        }

        // Calculate distances from COG to each attachment point
        let distances: Vec<f64> = self
                    .dead_legs
            .iter()
            .map(|leg| leg.attachment_point.coords.magnitude())
            .collect();

        // Check if all distances are approximately equal
        let first = distances[0];
        distances.iter().all(|&d| (d - first).abs() < 0.1)
    }

    /// Calculate distribution for asymmetric bridle using moment equilibrium
    fn calculate_asymmetric_distribution(&mut self) -> Result<BridleAnalysis, BridleError> {
        // This requires solving the static equilibrium equations
        // ΣF = 0 (sum of forces)
        // ΣM = 0 (sum of moments about COG)

        // For a general n-leg bridle, this is a system of equations:
        // T1*cos(θ1) + T2*cos(θ2) + ... + Tn*cos(θn) = W (vertical equilibrium)
        // And moment equations about X and Z axes

        // This requires matrix solution - implement numerical solver
        // For now, return error
        Err(BridleError::UnsupportedConfiguration(
            "Asymmetric bridle requires numerical solver (TODO)".into(),
        ))
    }

    /// Verify live leg distribution
    fn verify_live_leg_distribution(&self) -> Result<BridleAnalysis, BridleError> {
        let total_tension: f64 = self
                    .live_legs
            .iter()
            .map(|leg| leg.tension.get::<pound_force>())
            .sum();

        let load_lbf = self.load.get::<pound>();

        // Check if tensions sum to load (within tolerance)
        if (total_tension - load_lbf).abs() > 10.0 {
            return Err(BridleError::UnbalancedLoad {
                total_tension: DisplayForce(Force::new::<pound_force>(total_tension)),
                required_load: DisplayWeight(self.load),
            });
        }

        Ok(BridleAnalysis {
            total_load: self.load,
            dead_leg_tensions: Vec::new(),
            live_leg_tensions: self.live_legs.iter().map(|l| l.tension).collect(),
            is_balanced: true,
        })
    }

    /// Calculate mixed distribution (live + dead legs)
    fn calculate_mixed_distribution(&mut self) -> Result<BridleAnalysis, BridleError> {
        // Live legs provide their set tensions
        let live_tension_total: f64 = self
                    .live_legs
            .iter()
            .map(|leg| leg.tension.get::<pound_force>())
            .sum();

        // Remaining load goes to dead legs
        let remaining_load = self.load.get::<pound>() - live_tension_total;

        if remaining_load < 0.0 {
            return Err(BridleError::ExcessiveLiveTension {
                live_tension: DisplayForce(Force::new::<pound_force>(live_tension_total)),
                total_load: DisplayWeight(self.load),
            });
        }

        // Distribute remaining load to dead legs
        let num_dead_legs = self.dead_legs.len();
        let load_per_dead_leg = remaining_load / num_dead_legs as f64;

        for leg in &mut self.dead_legs {
            let attachment_world = self.load_cog + leg.attachment_point.coords;
            let leg_vector = self.hook_position - attachment_world;
            let leg_length = leg_vector.magnitude();
            let vertical_component = leg_vector.y;

            let angle_from_vertical =
                            Angle::new::<radian>((vertical_component / leg_length).acos());
                
            let tension = load_per_dead_leg / angle_from_vertical.get::<radian>().cos();
            leg.tension = Force::new::<pound_force>(tension);
        }

        // Verify all legs are safe
        for leg in &self.dead_legs {
            if !leg.sling.is_safe(leg.tension) {
                return Err(BridleError::LegOverCapacity {
                    leg_id: leg.sling.id.clone(),
                    tension: DisplayForce(leg.tension),
                    capacity: DisplayWeight(leg.sling.effective_capacity()),
                });
            }
        }

        Ok(BridleAnalysis {
            total_load: self.load,
            dead_leg_tensions: self.dead_legs.iter().map(|l| l.tension).collect(),
            live_leg_tensions: self.live_legs.iter().map(|l| l.tension).collect(),
            is_balanced: true,
        })
    }
}

#[derive(Debug)]
pub struct BridleAnalysis {
    pub total_load: Weight,
    pub dead_leg_tensions: Vec<Force>,
    pub live_leg_tensions: Vec<Force>,
    pub is_balanced: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum BridleError {
    #[error("No legs in bridle configuration")]
    NoLegs,

    #[error("Leg {leg_id} over capacity: 
        \n\ttension: {tension}
        \n\tcapacity: {capacity}")]
    LegOverCapacity {
        leg_id: String,
        tension: DisplayForce,
        capacity: DisplayWeight
    },

    #[error("Unbalanced load: total tension {total_tension} != required load {required_load}")]
    UnbalancedLoad {
        total_tension: DisplayForce,
        required_load: DisplayWeight
    },

    #[error("Live legs provide excessive tension: {live_tension} > total load {total_load}")]
    ExcessiveLiveTension {
        live_tension: DisplayForce,
        total_load: DisplayWeight,
    },

    #[error("Unsupported configuration: {0}")]
    UnsupportedConfiguration(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigging::slings::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_symmetric_4_leg_bridle() {
        let mut bridle = Bridle::new(
            Weight::new::<pound>(10000.0),
            na::Point3::origin(),
            na::Point3::new(0.0, 20.0, 0.0), // Hook 20 ft above load
        );

        // Add 4 symmetric legs at corners of square
        let corners = vec![
            na::Point3::new(5.0, 0.0, 5.0),
            na::Point3::new(-5.0, 0.0, 5.0),
            na::Point3::new(5.0, 0.0, -5.0),
            na::Point3::new(-5.0, 0.0, -5.0),
        ];

        for corner in corners {
            let sling = Sling::new(
                "Test",
                SlingMaterial::WireRope {
                    diameter: Distance::new::<inch>(0.5),
                    construction: WireRopeConstruction::SixByNineteen,
                },
                Weight::new::<pound>(5000.0),
                Distance::new::<foot>(25.0),
            );

            bridle.add_dead_leg(sling, corner);
        }

        let analysis = bridle.calculate_load_distribution().unwrap();

        // Each leg should carry 2500 lbs (10000 / 4)
        // Actual tension will be higher due to angle
        for tension in analysis.dead_leg_tensions {
            assert!(tension.get::<pound_force>() > 2500.0);
            assert!(tension.get::<pound_force>() < 5000.0);
        }
    }
}
