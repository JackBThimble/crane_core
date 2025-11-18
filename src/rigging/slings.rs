use crate::types::*;

/// Types of sling materials
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlingMaterial {
    /// Wire rope sling (most common for heavy lifts)
    WireRope {
        diameter: Length,
        construction: WireRopeConstruction,
    },
    
    /// Synthetic web sling (nylon, polyester)
    Synthetic {
        width: Length,
        plies: u32,
    },
    
    /// Chain sling (alloy steel)
    Chain {
        grade: ChainGrade,
        size: Length,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WireRopeConstruction {
    /// 6x19 construction (standard)
    SixByNineteen,
    
    /// 6x37 construction (more flexible)
    SixByThirtySeven,
    
    /// 7x19 construction (aircraft cable)
    SevenByNineteen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainGrade {
    /// Grade 80 alloy chain
    Grade80,
    
    /// Grade 100 alloy chain (higher strength)
    Grade100,
}

/// Hitch configuration - how the sling is rigged
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitchType {
    /// Vertical hitch - straight up and down (100% capacity)
    Vertical,
    
    /// Choker hitch - wrapped around load (efficiency ~75%)
    Choker,
    
    /// Basket hitch - sling goes under load and both ends lift (2x capacity if vertical)
    Basket {
        /// Angle from vertical for each leg
        sling_angle: Angle,
    },
    
    /// Bridle - multiple legs meeting at a single point
    Bridle {
        /// Number of legs
        num_legs: u32,
        
        /// Angle from vertical for each leg
        sling_angle: Angle,
    },
}

/// A rigging sling with its properties
#[derive(Debug, Clone)]
pub struct Sling {
    /// Sling identifier
    pub id: String,
    
    /// Material and construction
    pub material: SlingMaterial,
    
    /// Rated capacity (vertical hitch)
    pub rated_capacity: Mass,
    
    /// Length of sling
    pub length: Length,
    
    /// Current hitch configuration
    pub hitch: HitchType,
    
    /// Whether this is a "dead" leg (static) or "live" leg (adjustable)
    pub leg_type: LegType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LegType {
    /// Dead leg - static sling, load determined by geometry
    Dead,
    
    /// Live leg - adjustable tension (chain fall, lever hoist, etc.)
    Live {
        /// Current tension in the leg
        current_tension: Force,
        
        /// Maximum capacity of the live rigging device
        device_capacity: Mass,
    },
}

impl Sling {
    pub fn new(
        id: impl Into<String>,
        material: SlingMaterial,
        rated_capacity: Mass,
        length: Length,
    ) -> Self {
        Self {
            id: id.into(),
            material,
            rated_capacity,
            length,
            hitch: HitchType::Vertical,
            leg_type: LegType::Dead,
        }
    }
    
    /// Calculate effective capacity based on hitch type and angle
    /// 
    /// This is CRITICAL - sling angle dramatically affects capacity
    pub fn effective_capacity(&self) -> Mass {
        let base_capacity = self.rated_capacity.get::<pound>();
        
        let capacity_lbs = match self.hitch {
            HitchType::Vertical => base_capacity,
            
            HitchType::Choker => {
                // Choker hitch reduces capacity to ~75% due to stress concentration
                base_capacity * 0.75
            }
            
            HitchType::Basket { sling_angle } => {
                // Basket hitch: 2 legs supporting load
                // Each leg tension = Load / (2 * cos(angle))
                // Total capacity = 2 * base_capacity * cos(angle)
                let angle_rad = sling_angle.get::<radian>();
                let angle_factor = angle_rad.cos();
                
                // ASME B30.9 sling angle factors
                // Angle > 60° from vertical: significant capacity reduction
                2.0 * base_capacity * angle_factor
            }
            
            HitchType::Bridle { num_legs, sling_angle } => {
                // Multi-leg bridle
                // Each leg: tension = Load / (n * cos(angle))
                // Total capacity = n * base_capacity * cos(angle)
                let angle_rad = sling_angle.get::<radian>();
                let angle_factor = angle_rad.cos();
                
                num_legs as f64 * base_capacity * angle_factor
            }
        };
        
        Mass::new::<pound>(capacity_lbs)
    }
    
    /// Calculate tension in this sling for a given load
    /// 
    /// For dead legs, tension is determined purely by geometry
    /// For live legs, tension is set by the device
    pub fn calculate_tension(&self, load_share: Mass) -> Force {
        match self.leg_type {
            LegType::Dead => {
                // For dead legs, calculate tension from geometry
                let load = load_share.get::<pound>();
                
                let tension_lbf = match self.hitch {
                    HitchType::Vertical => load,
                    
                    HitchType::Choker => {
                        // Choker creates higher tension due to geometry
                        load / 0.75
                    }
                    
                    HitchType::Basket { sling_angle } => {
                        // Each leg tension = Load / (2 * cos(angle))
                        let angle_rad = sling_angle.get::<radian>();
                        load / (2.0 * angle_rad.cos())
                    }
                    
                    HitchType::Bridle { num_legs: _, sling_angle } => {
                        // Each leg tension = Load / (n * cos(angle))
                        let angle_rad = sling_angle.get::<radian>();
                        load / angle_rad.cos()
                    }
                };
                
                Force::new::<pound_force>(tension_lbf)
            }
            
            LegType::Live { current_tension, .. } => {
                // Live legs have their tension set directly
                current_tension
            }
        }
    }
    
    /// Check if this sling is safe for the given tension
    pub fn is_safe(&self, tension: Force) -> bool {
        let capacity = self.effective_capacity();
        let tension_as_weight = Mass::new::<pound>(tension.get::<pound_force>());
        
        tension_as_weight <= capacity
    }
    
    /// Calculate the sling angle given attachment geometry
    /// 
    /// angle = arccos(height / sling_length)
    pub fn calculate_sling_angle(
        &self,
        hook_height: Length,
        load_attachment_height: Length,
    ) -> Angle {
        let vertical_distance = (hook_height.get::<foot>() - load_attachment_height.get::<foot>()).abs();
        let sling_len = self.length.get::<foot>();
        
        if sling_len < 1e-6 {
            return Angle::new::<degree>(0.0);
        }
        
        // For basket or bridle, sling forms a triangle
        // cos(angle) = vertical_distance / sling_length
        let cos_angle = (vertical_distance / sling_len).min(1.0);
        Angle::new::<radian>(cos_angle.acos())
    }
}

/// ASME B30.9 sling angle factors
/// 
/// These are the standard capacity reduction factors based on sling angle
pub fn asme_angle_factor(angle_from_vertical: Angle) -> f64 {
    let degrees = angle_from_vertical.get::<degree>();
    
    // ASME B30.9 Table 9-1.1
    match degrees {
        d if d <= 5.0 => 1.000,
        d if d <= 15.0 => 0.966,
        d if d <= 30.0 => 0.866,
        d if d <= 45.0 => 0.707,
        d if d <= 60.0 => 0.500,
        _ => 0.0, // Angles > 60° not recommended
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_vertical_hitch_capacity() {
        let sling = Sling::new(
            "Test-1",
            SlingMaterial::WireRope {
                diameter: Length::new::<inch>(0.5),
                construction: WireRopeConstruction::SixByNineteen,
            },
            Mass::new::<pound>(5000.0),
            Length::new::<foot>(10.0),
        );
        
        let capacity = sling.effective_capacity();
        assert_relative_eq!(capacity.get::<pound>(), 5000.0);
    }
    
    #[test]
    fn test_choker_hitch_capacity() {
        let mut sling = Sling::new(
            "Test-2",
            SlingMaterial::WireRope {
                diameter: Length::new::<inch>(0.5),
                construction: WireRopeConstruction::SixByNineteen,
            },
            Mass::new::<pound>(5000.0),
            Length::new::<foot>(10.0),
        );
        
        sling.hitch = HitchType::Choker;
        
        let capacity = sling.effective_capacity();
        // Choker reduces to 75%
        assert_relative_eq!(capacity.get::<pound>(), 3750.0);
    }
    
    #[test]
    fn test_basket_hitch_vertical() {
        let mut sling = Sling::new(
            "Test-3",
            SlingMaterial::WireRope {
                diameter: Length::new::<inch>(0.5),
                construction: WireRopeConstruction::SixByNineteen,
            },
            Mass::new::<pound>(5000.0),
            Length::new::<foot>(10.0),
        );
        
        // Basket hitch at 0 degrees (vertical legs)
        sling.hitch = HitchType::Basket {
            sling_angle: Angle::new::<degree>(0.0),
        };
        
        let capacity = sling.effective_capacity();
        // 2 vertical legs = 2x capacity
        assert_relative_eq!(capacity.get::<pound>(), 10000.0);
    }
    
    #[test]
    fn test_basket_hitch_angled() {
        let mut sling = Sling::new(
            "Test-4",
            SlingMaterial::WireRope {
                diameter: Length::new::<inch>(0.5),
                construction: WireRopeConstruction::SixByNineteen,
            },
            Mass::new::<pound>(5000.0),
            Length::new::<foot>(10.0),
        );
        
        // Basket hitch at 30 degrees from vertical
        sling.hitch = HitchType::Basket {
            sling_angle: Angle::new::<degree>(30.0),
        };
        
        let capacity = sling.effective_capacity();
        // 2 legs * 5000 * cos(30°) = 2 * 5000 * 0.866 = 8660
        assert_relative_eq!(capacity.get::<pound>(), 8660.0, epsilon = 10.0);
    }
    
    #[test]
    fn test_bridle_tension_calculation() {
        let mut sling = Sling::new(
            "Test-5",
            SlingMaterial::WireRope {
                diameter: Length::new::<inch>(0.5),
                construction: WireRopeConstruction::SixByNineteen,
            },
            Mass::new::<pound>(5000.0),
            Length::new::<foot>(10.0),
        );
        
        // 4-leg bridle at 30 degrees
        sling.hitch = HitchType::Bridle {
            num_legs: 4,
            sling_angle: Angle::new::<degree>(30.0),
        };
        
        // 10,000 lb load shared by 4 legs
        let load_share = Mass::new::<pound>(10000.0 / 4.0);
        let tension = sling.calculate_tension(load_share);
        
        // Tension = 2500 / (cos(30°)) = 2500 / 0.866 = 2887 lbf
        assert_relative_eq!(
            tension.get::<pound_force>(),
            2887.0,
            epsilon = 10.0
        );
    }
    
        #[test]
    fn test_basket_tension_calculation_vertical() {
        let mut sling = Sling::new(
            "Test-6",
            SlingMaterial::Synthetic { 
                width: Length::new::<inch>(2.0), 
                plies: 2 },
            Mass::new::<pound>(3000.0),
            Length::new::<foot>(8.0),
        );
        sling.hitch = HitchType::Basket { sling_angle: Angle::new::<degree>(0.0) };
        let load_share = Mass::new::<pound>(4000.0);
        let tension = sling.calculate_tension(load_share);
        assert_relative_eq!(tension.get::<pound_force>(), 2000.0, epsilon = 1.0);
    }
    
    #[test]
    fn test_basket_tension_calculation_low_angle() {
        let mut sling = Sling::new(
            "Test-6",
            SlingMaterial::Synthetic { 
                width: Length::new::<inch>(2.0), 
                plies: 2 },
            Mass::new::<pound>(3000.0),
            Length::new::<foot>(8.0),
        );
        sling.hitch = HitchType::Basket { sling_angle: Angle::new::<degree>(30.0) };
        let load_share = Mass::new::<pound>(4000.0);
        let tension = sling.calculate_tension(load_share);
        assert_relative_eq!(tension.get::<pound_force>(), 2309.0, epsilon = 1.0);
    }
    
        #[test]
    fn test_basket_tension_calculation_high_angle() {
        let mut sling = Sling::new(
            "Test-7",
            SlingMaterial::Synthetic { 
                width: Length::new::<inch>(2.0), 
                plies: 2 },
            Mass::new::<pound>(3000.0),
            Length::new::<foot>(8.0),
        );
        sling.hitch = HitchType::Basket { sling_angle: Angle::new::<degree>(60.0) };
        let load_share = Mass::new::<pound>(4000.0);
        let tension = sling.calculate_tension(load_share);
        assert_relative_eq!(tension.get::<pound_force>(), 4000.0, epsilon = 1.0);
    }
    #[test]
    fn test_asme_angle_factors() {
        assert_relative_eq!(asme_angle_factor(Angle::new::<degree>(0.0)), 1.000);
        assert_relative_eq!(asme_angle_factor(Angle::new::<degree>(30.0)), 0.866, epsilon = 0.001);
        assert_relative_eq!(asme_angle_factor(Angle::new::<degree>(45.0)), 0.707, epsilon = 0.001);
        assert_relative_eq!(asme_angle_factor(Angle::new::<degree>(60.0)), 0.500);
    }
}
