//! Ground bearing pressure analysis for crane stability
//!
//! # Coordinate system
//!
//! All spatial calculations use a right-handed coordinate system:
//! - **X-axis**: Left(-) / Right(+) - lateral from crane centerline
//! - **Y-axis**: Down(-) / Up(+) - vertical from ground
//! - **Z-axis**: Rear(-) / Front(+) - longitudinal from crane center
//!
//! Origin is at ground level, centered on crane base.
//!
//!
//! # Internal units
//!
//! All `Point3` and `Vector3` coordinates are stored in **FEET**.
//! This is consistent throughout crane-core for performacne with nalgebra.
//!
//! Public APIs accept UOM types and convert at boundaries

use crate::types::*;
use nalgebra as na;


/// Ground bearing pressure calculation and validation
#[derive(Debug, Clone)]
pub struct GroundBearingAnalysis {
    /// Support points (outrigger positions or crawler track contact)
    pub support_points: Vec<SupportPoint>,

    /// Total crane weight (excluding load)
    pub crane_weight: Weight,

    /// Crane center of gravity (in crane coordinate system)
    pub crane_cog: na::Point3<f64>,

    /// Load weight
    pub load_weight: Weight,

    /// Load position (hook position)
    pub load_position: na::Point3<f64>,
}

/// A support point (outrigger or track content contact point)
#[derive(Debug, Clone)]
pub struct SupportPoint {
    /// Position in crane coordinates (feet)
    pub position: na::Point3<f64>,

    /// Contact area (pad or mat area)
    pub contact_area: Area,

    /// Name/identifier
    pub name: String,
}

/// Result of ground bearing analysis
#[derive(Debug, Clone)]
pub struct GroundBearingResult {
    /// Reaction force at each support point
    pub reactions: Vec<SupportReaction>,

    /// Maximum reaction force
    pub max_reaction: Force,

    /// Maximum ground pressure
    pub max_pressure: Pressure,

    /// Index of most loaded support
    pub critical_support_index: usize,
}

#[derive(Debug, Clone)]
pub struct SupportReaction {
    pub name: String,
    pub force: Force,
    pub pressure: Pressure,
    pub contact_area: Area,
}

#[derive(Debug, thiserror::Error)]
pub enum GroundBearingError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Unstable: negative reaction at support {0}")]
    UnstableConfiguration(String),

    #[error("Insufficient support points (need at least 3)")]
    InsufficientSupports,

    #[error("Ground pressure {actual} exceeds allowable {allowable}")]
    ExceedsAllowable {
        actual: DisplayGroundBearingPressure,
        allowable: DisplayGroundBearingPressure,
    },
}

impl GroundBearingAnalysis {
    /// Create new analysis with raw coordinates (internal use)
    ///
    /// # Arguments
    /// * `crane_weight` - Total crane weight
    /// * `crane_cog` - Crane center of gravity
    /// * `load_weight` - Load weight
    /// * `load_position` - Load position
    pub fn new(
        crane_weight: Weight,
        crane_cog: (Distance, Distance, Distance),
        load_weight: Weight,
        load_position: (Distance, Distance, Distance),
    ) -> Self {
        let (cx, cy, cz) = crane_cog;
        let (lx, ly, lz) = load_position;

        Self::new_na(
            crane_weight,
            point_from_distances(cx, cy, cz),
            load_weight,
            point_from_distances(lx, ly, lz),
        )
    }

    /// Create new anaysis using Point3 for raw coordinates (internal use)
    ///
    /// # Arguments
    /// * `crane_weight` - Total crane weight
    /// * `crane_cog` - Crane center of gravity (feet)
    /// * `load_weight` - Load weight
    /// * `load_position` - Load position (feet)
    pub fn new_na(
        crane_weight: Weight,
        crane_cog: na::Point3<f64>,
        load_weight: Weight,
        load_position: na::Point3<f64>,
    ) -> Self {
        Self {
            support_points: Vec::new(),
            crane_weight,
            crane_cog,
            load_weight,
            load_position,
        }
    }

    /// Add a support point (outrigger)
    pub fn add_support(
        &mut self,
        name: impl Into<String>,
        x: Distance,
        y: Distance,
        z: Distance,
        contact_area: Area,
    ) {
        self.add_support_na(
            name, 
            point_from_distances(x, y, z),
            contact_area,
        );
    }

    /// Add a support point (outrigger)
    ///
    /// # Arguments
    /// * `name` - Name of the support
    /// * `position` - position of outrigger
    /// * `contact_area` - Contact area of support point
    pub fn add_support_na(
        &mut self,
        name: impl Into<String>,
        position: na::Point3<f64>,
        contact_area: Area,
    ) {
        self.support_points.push(SupportPoint {
            position,
            contact_area,
            name: name.into(),
        });
    }

    /// Get crane COG
    pub fn crane_cog(&self) -> (Distance, Distance, Distance) {
        (
            from_coord(self.crane_cog.x),
            from_coord(self.crane_cog.y),
            from_coord(self.crane_cog.z),
        )
    }

    /// Calculate reactions at all support points
    pub fn calculate_reactions(&self) -> Result<GroundBearingResult, GroundBearingError> {
        if self.support_points.len() < 3 {
            return Err(GroundBearingError::InsufficientSupports);
        }

        // For 4-point support (most common case), use simplified analytical solution 
        if self.support_points.len() == 4 {
            return self.calculate_four_point_reactions();
        }

        // For other cases, use general method
        self.calculate_general_reactions()
    }

    /// Simplified calculations for 4 outriggers (most common case)
    fn calculate_four_point_reactions(&self) -> Result<GroundBearingResult, GroundBearingError> {
        let total_weight = self.crane_weight.get::<pound>() + self.load_weight.get::<pound>();

        let crane_moment = self.crane_cog.coords * self.crane_weight.get::<pound>();
        let load_moment = self.load_position.coords * self.load_weight.get::<pound>();
        
        let combined_cog = (crane_moment + load_moment) / total_weight;

        let reactions = self.calculate_reactions_from_moments(&combined_cog, total_weight)?;

        // Calculate pressures
        let mut support_reactions = Vec::new();
        let mut max_reaction = Force::new::<pound_force>(0.0);
        let mut max_pressure = Pressure::new::<psi>(0.0);
        let mut critical_idx = 0;

        for (i, (support, &reaction_lb)) in self.support_points.iter().zip(reactions.iter()).enumerate() {
            let reaction = Force::new::<pound_force>(reaction_lb);
            let area_sq_in = support.contact_area.get::<square_inch>();
            let pressure = Pressure::new::<psi>(reaction_lb / area_sq_in);

            if reaction > max_reaction {
                max_reaction = reaction;
                max_pressure = pressure;
                critical_idx = i;
            }

            support_reactions.push(SupportReaction {
                name: support.name.clone(),
                force: reaction,
                pressure,
                contact_area: support.contact_area,
            });
        }

        Ok(GroundBearingResult { 
            reactions: support_reactions,
            max_reaction,
            max_pressure,
            critical_support_index: critical_idx,

        })
    }

    /// Calculate reactions based on moment equilibrium
    fn calculate_reactions_from_moments(
        &self,
        combined_cog: &na::Vector3<f64>,
        total_weight: f64,
    ) -> Result<Vec<f64>, GroundBearingError> {
        if self.support_points.len() != 4 {
            return Err(GroundBearingError::InvalidConfiguration(
                "Four-point method requires exactly 4 supports".into()
            ));
        }

        let positions: Vec<na::Vector3<f64>> = self.support_points
            .iter()
            .map(|s| s.position.coords)
            .collect();

        // Calculate centerline (average position)
        let avg_x = positions.iter().map(|p| p.x).sum::<f64>() / 4.0;
        let avg_z = positions.iter().map(|p| p.z).sum::<f64>() / 4.0;

        // Calculate moment arms (feet)
        let dx = combined_cog.x - avg_x;
        let dz = combined_cog.z - avg_z;

        // Base reaction if centered
        let base_reaction = total_weight / 4.0;

        // Calculate span between outriggers (feet)
        let x_span = positions.iter().map(|p| p.x).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            - positions.iter().map(|p| p.x).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let z_span = positions.iter().map(|p| p.z).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            - positions.iter().map(|p| p.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // Moment effects on each outrigger
        let mut reactions = vec![base_reaction; 4];

        for (i, pos) in positions.iter().enumerate() {
            let x_effect = if pos.x > avg_x {
        (dx / x_span) * total_weight / 2.0
            } else {
                -(dx / x_span) * total_weight / 2.0
            };

            let z_effect = if pos.z > avg_z {
                (dz / z_span) * total_weight / 2.0
            } else {
                -(dz / z_span) * total_weight / 2.0
            };

            reactions[i] += x_effect + z_effect;

            // Check for negative reaction (tipping)
            if reactions[i] < 0.0 {
                return Err(GroundBearingError::UnstableConfiguration(
                    self.support_points[i].name.clone()
                ));
            }
        }
        Ok(reactions)

    }

    /// General method for any number of supports
    fn calculate_general_reactions(&self) -> Result<GroundBearingResult, GroundBearingError> {
        // Conservative approach: assume worst-case loading
        let total_weight = self.crane_weight.get::<pound>() + self.load_weight.get::<pound>();
        let worst_case_reaction = Force::new::<pound_force>(total_weight);

        let mut critical_idx = 0;
        let mut min_distance = f64::MAX;

        for (i, support) in self.support_points.iter().enumerate() {
            let distance = (support.position - self.load_position).norm();
            if distance < min_distance {
                min_distance = distance;
                critical_idx = i;
            }
        }

        let critical_support = &self.support_points[critical_idx];
        let area_sq_in = critical_support.contact_area.get::<square_inch>();
        let worst_pressure = Pressure::new::<psi>(total_weight / area_sq_in);

        let mut support_reactions = Vec::new();
        for (i, support) in self.support_points.iter().enumerate() {
            let reaction = if i == critical_idx {
                worst_case_reaction
            } else {
                Force::new::<pound_force>(0.0)
            };

            let area = support.contact_area.get::<uom::si::area::square_inch>();
            let pressure = Pressure::new::<psi>(reaction.get::<pound_force>() / area);
            support_reactions.push(SupportReaction {
                name: support.name.clone(),
                force: reaction,
                pressure,
                contact_area: support.contact_area,
            });
        }

        Ok(GroundBearingResult {
            reactions: support_reactions,
            max_reaction: worst_case_reaction,
            max_pressure: worst_pressure,
            critical_support_index: critical_idx,
        })
    }

    /// Validate against allowable soid bearing pressure
    pub fn validate_soil_capacity(
        &self, 
        allowable_pressure: Pressure,
    ) -> Result<(), GroundBearingError> {
        let result = self.calculate_reactions()?;

        let max_psi = result.max_pressure.get::<psi>();
        let allowable_psi = allowable_pressure.get::<psi>();

        if max_psi > allowable_psi {
            return Err(GroundBearingError::ExceedsAllowable {
                actual: DisplayGroundBearingPressure(result.max_pressure),
                allowable: DisplayGroundBearingPressure(allowable_pressure)
            });
        } 
        Ok(())
    }

    /// Calculate required mat area for given soil capacity
    pub fn required_mat_area(
        &self,
        allowable_pressure: Pressure,
        safety_factor: f64,
    ) -> Result<Area, GroundBearingError> {
        let result = self.calculate_reactions()?;
        let max_reaction_lb = result.max_reaction.get::<pound_force>();
        let allowable_psi = allowable_pressure.get::<psi>() / safety_factor;

        let required_sq_in = max_reaction_lb / allowable_psi;

        Ok(Area::new::<square_inch>(required_sq_in))
    }
}

impl GroundBearingResult {
    /// Format results for display
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("Ground Bearing Analysis:\n");
        s.push_str(&format!("\nCritical Support: {}\n",
            self.reactions[self.critical_support_index].name));
        s.push_str(&format!(" Max Reaction: {:.0} lbs\n",
            self.max_reaction.get::<pound_force>()));
        s.push_str(&format!(" Max Pressure: {:.1} PSI\n", 
            self.max_pressure.get::<psi>()));

        s.push_str("\nAll Supports:\n");
        for reaction in &self.reactions {
            s.push_str(&format!(" {}: {:.0} lbs ({:.1} PSI over {:.1} sq ft)\n", 
                reaction.name,
                reaction.force.get::<pound_force>(),
                reaction.pressure.get::<psi>(),
                reaction.contact_area.get::<square_foot>(),
            ));
        }
        s
    }
}

/// Common soil bearing capabilities
pub mod soil_capacities {
    use crate::types::*;

    pub fn soft_clay() -> Pressure {
        Pressure::new::<psi>(10.0)
    }

    pub fn medium_clay() -> Pressure {
        Pressure::new::<psi>(25.0)
    }

    pub fn stiff_clay() -> Pressure {
        Pressure::new::<psi>(40.0)
    }

    pub fn loose_sand() -> Pressure {
        Pressure::new::<psi>(20.0)
    }

    pub fn dense_sand() -> Pressure {
        Pressure::new::<psi>(50.0)
    }

    pub fn gravel() -> Pressure {
        Pressure::new::<psi>(80.0)
    }

    pub fn soft_rock() -> Pressure {
        Pressure::new::<psi>(150.0)
    }

    pub fn hard_rock() -> Pressure {
        Pressure::new::<psi>(300.0)
    }

    pub fn paved_surface() -> Pressure {
        Pressure::new::<psi>(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    #[test]
    fn test_centered_load() {
        let mut analysis = GroundBearingAnalysis::new(
            Weight::new::<pound>(100000.0),
(Distance::new::<foot>(0.0), Distance::new::<foot>(5.0), Distance::new::<foot>(0.0)),
            Weight::new::<pound>(50000.0),
(Distance::new::<foot>(0.0), Distance::new::<foot>(50.0), Distance::new::<foot>(0.0)),
        );

        let pad_area = Area::new::<square_foot>(4.0);
        analysis.add_support("FL", Distance::new::<foot>(-10.0), Distance::new::<foot>(0.0), Distance::new::<foot>(10.0), pad_area);
        analysis.add_support("FR", Distance::new::<foot>(10.0), Distance::new::<foot>(0.0), Distance::new::<foot>(10.0), pad_area);
        analysis.add_support("RR", Distance::new::<foot>(-10.0), Distance::new::<foot>(0.0), Distance::new::<foot>(-10.0), pad_area);
        analysis.add_support("RL", Distance::new::<foot>(10.0), Distance::new::<foot>(0.0), Distance::new::<foot>(-10.0), pad_area);

        let result = analysis.calculate_reactions().unwrap();

        for reaction in &result.reactions {
            let force_lb = reaction.force.get::<pound_force>();
            assert_relative_eq!(force_lb, 37500.0, epsilon = 5000.0);
        }
    }

}


