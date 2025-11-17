use nalgebra as na;
use crate::equipment::crane::{Crane, LiftError};
use crate::types::*;

/// A tandem lift configuration with multiple cranes sharing a load
/// 
/// Per ASME B30.5, tandem lifts require:
/// - Detailed lift plan
/// - Qualified lift director
/// - Load distribution calculation
/// - Stability analysis for each crane
/// - 75% capacity limit for each crane (25% safety margin)
#[derive(Debug)]
pub struct TandemLift<C: Crane> {
    /// Cranes participating in the lift
    pub cranes: Vec<TandemCrane<C>>,
    
    /// Total load being lifted
    pub total_load: Weight,
    
    /// Load center of gravity
    pub load_cog: na::Point3<f64>,
    
    /// Rigging configuration
    pub rigging: TandemRigging,
    
    /// Safety factor (ASME requires 0.75 for tandem, meaning 75% of chart capacity)
    pub capacity_factor: f64,
}

#[derive(Debug)]
pub struct TandemCrane<C: Crane> {
    /// The crane itself
    pub crane: C,
    
    /// Hook position for this crane
    pub hook_position: na::Point3<f64>,
    
    /// Load share percentage (0.0 to 1.0)
    /// This will be calculated based on geometry
    pub load_share: f64,
}

#[derive(Debug, Clone)]
pub struct TandemRigging {
    /// Rigging attachment points on the load
    pub attachment_points: Vec<na::Point3<f64>>,
    
    /// Type of rigging configuration
    pub config_type: TandemRiggingType,
}

#[derive(Debug, Clone, Copy)]
pub enum TandemRiggingType {
    /// Direct lift - each crane picks directly on load
    Direct,
    
    /// Spreader beam - load distributed through beam
    SpreaderBeam {
        beam_weight: Weight,
        beam_length: Distance,
    },
    
    /// Equalizer beam - automatic load sharing
    EqualizerBeam {
        beam_weight: Weight,
    },
}

impl<C: Crane> TandemLift<C> {
    pub fn new(total_load: Weight, load_cog: na::Point3<f64>) -> Self {
        Self {
            cranes: Vec::new(),
            total_load,
            load_cog,
            rigging: TandemRigging {
                attachment_points: Vec::new(),
                config_type: TandemRiggingType::Direct,
            },
            capacity_factor: 0.75, // ASME B30.5 requirement
        }
    }
    
    /// Add a crane to the tandem lift
    pub fn add_crane(&mut self, crane: C, hook_position: na::Point3<f64>) {
        self.cranes.push(TandemCrane {
            crane,
            hook_position,
            load_share: 0.0, // Will be calculated
        });
    }
    
    /// Calculate load distribution between cranes
    /// 
    /// This is NOT always 50/50! It depends on:
    /// - Hook positions relative to load COG
    /// - Rigging geometry
    /// - Boom angles and deflection
    pub fn calculate_load_distribution(&mut self) -> Result<(), TandemLiftError> {
        if self.cranes.len() < 2 {
            return Err(TandemLiftError::InsufficientCranes);
        }
        
        match self.rigging.config_type {
            TandemRiggingType::Direct => {
                self.calculate_direct_distribution()
            }
            TandemRiggingType::SpreaderBeam { .. } => {
                self.calculate_spreader_distribution()
            }
            TandemRiggingType::EqualizerBeam { .. } => {
                self.calculate_equalizer_distribution()
            }
        }
    }
    
    /// Calculate load distribution for direct rigging
    /// 
    /// Uses moment equilibrium about load COG
    fn calculate_direct_distribution(&mut self) -> Result<(), TandemLiftError> {
        if self.cranes.len() != 2 {
            return Err(TandemLiftError::UnsupportedConfiguration(
                "Direct rigging only supports 2-crane tandem".into()
            ));
        }
        
        let hook1 = self.cranes[0].hook_position;
        let hook2 = self.cranes[1].hook_position;
        let cog = self.load_cog;
        
        // Distance from each hook to load COG (in horizontal plane)
        let d1 = ((hook1.x - cog.x).powi(2) + (hook1.z - cog.z).powi(2)).sqrt();
        let d2 = ((hook2.x - cog.x).powi(2) + (hook2.z - cog.z).powi(2)).sqrt();
        
        let total_distance = d1 + d2;
        
        if total_distance < 0.01 {
            return Err(TandemLiftError::InvalidGeometry(
                "Hooks too close together".into()
            ));
        }
        
        // Load share inversely proportional to distance from COG
        // Closer crane carries more load
        self.cranes[0].load_share = d2 / total_distance;
        self.cranes[1].load_share = d1 / total_distance;
        
        Ok(())
    }
    
    /// Calculate load distribution with spreader beam
    /// 
    /// Spreader beam distributes load based on attachment geometry
    fn calculate_spreader_distribution(&mut self) -> Result<(), TandemLiftError> {
        // For spreader beam, calculate reactions at beam pickup points
        // This involves solving beam equilibrium equations
        
        // Simplified: assume symmetric loading
        let share_per_crane = 1.0 / self.cranes.len() as f64;
        for crane in &mut self.cranes {
            crane.load_share = share_per_crane;
        }
        
        Ok(())
    }
    
    /// Calculate load distribution with equalizer beam
    /// 
    /// Equalizer beams automatically balance loads (theoretically)
    fn calculate_equalizer_distribution(&mut self) -> Result<(), TandemLiftError> {
        // Equalizer beams should distribute evenly
        // BUT: in practice, geometry and deflection affect this
        let share_per_crane = 1.0 / self.cranes.len() as f64;
        for crane in &mut self.cranes {
            crane.load_share = share_per_crane;
        }
        
        Ok(())
    }
    
    /// Validate entire tandem lift configuration
    pub fn validate(&mut self) -> Result<TandemLiftAnalysis, TandemLiftError> {
        // Calculate load distribution
        self.calculate_load_distribution()?;
        
        let mut crane_analyses = Vec::new();
        
        // Validate each crane
        for tandem_crane in &self.cranes {
            let crane_load = Weight::new::<pound>(
                self.total_load.get::<pound>() * tandem_crane.load_share
            );
            
            // Get crane's rated capacity
            let rated_capacity = tandem_crane.crane.rated_capacity();
            
            // Apply tandem capacity factor (75% of chart)
            let allowed_capacity = Weight::new::<pound>(
                rated_capacity.get::<pound>() * self.capacity_factor
            );
            
            // Check if crane is within capacity
            if crane_load > allowed_capacity {
                return Err(TandemLiftError::CraneOverCapacity {
                    crane_index: crane_analyses.len(),
                    load: DisplayWeight(crane_load),
                    allowed: DisplayWeight(allowed_capacity),
                });
            }
            
            // Validate individual crane stability
            tandem_crane.crane.validate_lift(crane_load)?;
            
            crane_analyses.push(CraneAnalysis {
                load_share: tandem_crane.load_share,
                crane_load,
                rated_capacity,
                allowed_capacity,
                utilization: crane_load.get::<pound>() / allowed_capacity.get::<pound>(),
            });
        }
        
        Ok(TandemLiftAnalysis {
            total_load: self.total_load,
            crane_analyses,
            is_valid: true,
        })
    }
}

#[derive(Debug)]
pub struct TandemLiftAnalysis {
    pub total_load: Weight,
    pub crane_analyses: Vec<CraneAnalysis>,
    pub is_valid: bool,
}

#[derive(Debug)]
pub struct CraneAnalysis {
    /// Percentage of total load (0.0 to 1.0)
    pub load_share: f64,
    
    /// Actual load on this crane
    pub crane_load: Weight,
    
    /// Crane's rated capacity at current config
    pub rated_capacity: Weight,
    
    /// Allowed capacity with tandem safety factor
    pub allowed_capacity: Weight,
    
    /// Utilization ratio (actual / allowed)
    pub utilization: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum TandemLiftError {
    #[error("Insufficient cranes for tandem lift (need at least 2)")]
    InsufficientCranes,
    
    #[error("Unsupported configuration: {0}")]
    UnsupportedConfiguration(String),
    
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    
    #[error("Crane {crane_index} over capacity: load {load} exceeds allowed {allowed}")]
    CraneOverCapacity {
        crane_index: usize,
        load: DisplayWeight,
        allowed: DisplayWeight,
    },
    
    #[error("Crane validation failed: {0}")]
    CraneValidation(#[from] LiftError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::crane::MobileCrane;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_two_crane_load_distribution() {
        let mut tandem = TandemLift::new(
            Weight::new::<pound>(100000.0),
            na::Point3::new(50.0, 10.0, 0.0), // Load COG
        );
        
        // Crane 1 at origin (50 ft from load)
        let crane1 = MobileCrane::new(
            "Grove",
            "GMK5250L",
            Distance::new::<foot>(100.0),
            Distance::new::<foot>(10.0),
        );
        tandem.add_crane(crane1, na::Point3::new(0.0, 10.0, 0.0));
        
        // Crane 2 at 100 ft (50 ft from load on other side)
        let crane2 = MobileCrane::new(
            "Grove",
            "GMK5250L",
            Distance::new::<foot>(100.0),
            Distance::new::<foot>(10.0),
        );
        tandem.add_crane(crane2, na::Point3::new(100.0, 10.0, 0.0));
        
        // Calculate distribution
        tandem.calculate_load_distribution().unwrap();
        
        // Load COG is centered, so each crane should carry 50%
        assert_relative_eq!(tandem.cranes[0].load_share, 0.5, epsilon = 0.01);
        assert_relative_eq!(tandem.cranes[1].load_share, 0.5, epsilon = 0.01);
    }
    
    #[test]
    fn test_asymmetric_load_distribution() {
        let mut tandem = TandemLift::new(
            Weight::new::<pound>(100000.0),
            na::Point3::new(30.0, 10.0, 0.0), // Load COG offset
        );
        
        // Crane 1 at origin (30 ft from load)
        let crane1 = MobileCrane::new(
            "Grove",
            "GMK5250L",
            Distance::new::<foot>(100.0),
            Distance::new::<foot>(10.0),
        );
        tandem.add_crane(crane1, na::Point3::new(0.0, 10.0, 0.0));
        
        // Crane 2 at 100 ft (70 ft from load)
        let crane2 = MobileCrane::new(
            "Grove",
            "GMK5250L",
            Distance::new::<foot>(100.0),
            Distance::new::<foot>(10.0),
        );
        tandem.add_crane(crane2, na::Point3::new(100.0, 10.0, 0.0));
        
        tandem.calculate_load_distribution().unwrap();
        
        // Crane 1 is closer, so carries MORE load (70%)
        // Crane 2 is farther, so carries LESS load (30%)
        assert_relative_eq!(tandem.cranes[0].load_share, 0.7, epsilon = 0.01);
        assert_relative_eq!(tandem.cranes[1].load_share, 0.3, epsilon = 0.01);
    }
}
