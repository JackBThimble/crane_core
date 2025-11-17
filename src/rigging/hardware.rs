use crate::types::*;
use crate::types::units::*;

/// Types of shackles
#[derive(Debug, Clone, PartialEq)]
pub enum ShackleType {
    /// Anchor shackle (D-shackle) - wider body, better for multiple connections
    Anchor {
        size: Distance,
        pin_type: ShacklePinType,
    },
    
    /// Bow shackle (round shackle) - longer reach, better for angled loads
    Bow {
        size: Distance,
        pin_type: ShacklePinType,
    },
    
    /// Chain shackle - designed for permanent chain connections
    Chain {
        size: Distance,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShacklePinType {
    /// Screw pin (quick release, lower capacity)
    ScrewPin,
    
    /// Bolt type with nut and cotter pin (higher capacity, more secure)
    BoltType,
    
    /// Safety bolt with recessed allen head
    SafetyBolt,
}

/// Rigging hardware component
#[derive(Debug, Clone)]
pub struct Hardware {
    pub id: String,
    pub hardware_type: HardwareType,
    pub rated_capacity: Weight,
    pub material: HardwareMaterial,
    pub manufacturer: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareType {
    Shackle(ShackleType),
    Hook(HookType),
    MasterLink(MasterLinkType),
    Turnbuckle(TurnbuckleType),
    EyeBolt(EyeBoltType),
    QuickLink(QuickLinkType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookType {
    /// Eye hook - simple hook with eye for attachment
    Eye {
        throat_opening: Distance,
        has_latch: bool,
    },
    
    /// Grab hook - for chain, has narrower throat
    Grab {
        throat_opening: Distance,
    },
    
    /// Sorting hook - wide opening for material handling
    Sorting {
        throat_opening: Distance,
    },
    
    /// Swivel hook - rotates to prevent twisting
    Swivel {
        throat_opening: Distance,
        has_latch: bool,
    },
    
    /// Foundry hook - heavy duty, typically no latch
    Foundry {
        throat_opening: Distance,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MasterLinkType {
    /// Forged master link - multiple attachment points
    Forged {
        num_attachments: u32,
    },
    
    /// Hammerlok master link - connects sling eyes to crane hook
    Hammerlok,
    
    /// Oblong master link - oval shape
    Oblong,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnbuckleType {
    /// Hook and hook
    HookHook,
    
    /// Hook and eye
    HookEye,
    
    /// Eye and eye
    EyeEye,
    
    /// Jaw and jaw
    JawJaw,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EyeBoltType {
    /// Regular eye bolt - VERTICAL LOAD ONLY
    Regular {
        thread_diameter: Distance,
    },
    
    /// Shoulder eye bolt - can handle angular loads
    Shoulder {
        thread_diameter: Distance,
    },
    
    /// Swivel eye bolt - rotates
    Swivel {
        thread_diameter: Distance,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuickLinkType {
    /// Screw pin quick link
    ScrewPin,
    
    /// Bolt type quick link
    BoltType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HardwareMaterial {
    /// Alloy steel - most common for heavy lifting
    AlloySteel {
        grade: SteelGrade,
    },
    
    /// Carbon steel
    CarbonSteel,
    
    /// Stainless steel - corrosion resistant, lower capacity
    StainlessSteel {
        grade: StainlessGrade,
    },
    
    /// Galvanized steel
    Galvanized,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SteelGrade {
    /// Grade 80 alloy
    Grade80,
    
    /// Grade 100 alloy (higher strength)
    Grade100,
    
    /// Grade 120 alloy (highest strength)
    Grade120,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StainlessGrade {
    /// 304 stainless
    SS304,
    
    /// 316 stainless (better corrosion resistance)
    SS316,
}

impl Hardware {
    pub fn new(
        id: impl Into<String>,
        hardware_type: HardwareType,
        rated_capacity: Weight,
        material: HardwareMaterial,
    ) -> Self {
        Self {
            id: id.into(),
            hardware_type,
            rated_capacity,
            material,
            manufacturer: "Generic".into(),
        }
    }
    
    /// Calculate effective capacity based on loading conditions
    /// 
    /// CRITICAL: Side loading, angular loading, and improper use can
    /// dramatically reduce hardware capacity
    pub fn effective_capacity(&self, loading: LoadingCondition) -> Weight {
        let base_capacity = self.rated_capacity.get::<pound>();
        
        let capacity_factor = match (&self.hardware_type, loading) {
            // Shackles
            (HardwareType::Shackle(_), LoadingCondition::InLine) => 1.0,
            (HardwareType::Shackle(_), LoadingCondition::SideLoad { angle }) => {
                // Side loading dramatically reduces capacity
                self.shackle_side_load_factor(angle)
            }
            
            // Eye bolts - CRITICAL: Regular eye bolts are VERTICAL ONLY
            (HardwareType::EyeBolt(EyeBoltType::Regular { .. }), LoadingCondition::InLine) => 1.0,
            (HardwareType::EyeBolt(EyeBoltType::Regular { .. }), LoadingCondition::Angular { .. }) => {
                // Regular eye bolts should NEVER be loaded at an angle
                0.0 // Zero capacity - unsafe!
            }
            (HardwareType::EyeBolt(EyeBoltType::Shoulder { .. }), LoadingCondition::Angular { angle }) => {
                self.eye_bolt_angular_factor(angle)
            }
            
            // Hooks - generally in-line loading
            (HardwareType::Hook(_), LoadingCondition::InLine) => 1.0,
            (HardwareType::Hook(_), LoadingCondition::PointLoad) => {
                // Point loading on hook tip reduces capacity
                0.5
            }
            
            // Master links
            (HardwareType::MasterLink(_), LoadingCondition::InLine) => 1.0,
            
            // Turnbuckles
            (HardwareType::Turnbuckle(_), LoadingCondition::InLine) => 1.0,
            (HardwareType::Turnbuckle(_), LoadingCondition::SideLoad { .. }) => 0.5,
            
            // Quick links
            (HardwareType::QuickLink(_), LoadingCondition::InLine) => 1.0,
            (HardwareType::QuickLink(_), LoadingCondition::SideLoad { .. }) => 0.3,
            
            // Default conservative case
            _ => 0.5,
        };
        
        Weight::new::<pound>(base_capacity * capacity_factor)
    }
    
    /// Shackle side load reduction factor
    /// 
    /// Per ASME B30.26, side loading reduces shackle capacity significantly
    fn shackle_side_load_factor(&self, angle: Angle) -> f64 {
        let degrees = angle.get::<degree>();
        
        // ASME B30.26 side load factors
        match degrees {
            d if d <= 0.0 => 1.0,   // In-line
            d if d <= 45.0 => 0.7,  // Light side load - 30% reduction
            d if d <= 90.0 => 0.5,  // 90° side load - 50% reduction
            _ => 0.3,               // Severe side load - 70% reduction
        }
    }
    
    /// Eye bolt angular load factor
    /// 
    /// Shoulder eye bolts can handle angular loads, regular cannot
    fn eye_bolt_angular_factor(&self, angle: Angle) -> f64 {
        let degrees = angle.get::<degree>();
        
        // Angular loading reduction for shoulder eye bolts
        match degrees {
            d if d <= 0.0 => 1.0,   // Vertical
            d if d <= 45.0 => 0.7,  // Up to 45° - 30% reduction
            d if d <= 90.0 => 0.25, // Up to 90° - 75% reduction
            _ => 0.0,               // Beyond 90° - unsafe
        }
    }
    
    /// Check if this hardware is safe for the given load and conditions
    pub fn is_safe(&self, load: Weight, loading: LoadingCondition) -> bool {
        load <= self.effective_capacity(loading)
    }
}

/// Loading conditions that affect hardware capacity
#[derive(Debug, Clone, Copy)]
pub enum LoadingCondition {
    /// In-line loading (ideal)
    InLine,
    
    /// Side loading (reduces capacity)
    SideLoad {
        angle: Angle, // Angle from in-line
    },
    
    /// Angular loading (eye bolts)
    Angular {
        angle: Angle, // Angle from vertical
    },
    
    /// Point loading (hooks)
    PointLoad,
}

/// Standard shackle specifications from Crosby
pub mod crosby_shackles {
    use super::*;
    
    /// 1/4" G-209 screw pin anchor shackle
    pub fn quarter_inch_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-1/4".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(0.25),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(1000.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 3/8" G-209 screw pin anchor shackle
    pub fn three_eighths_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-3/8".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(0.375),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(2000.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 1/2" G-209 screw pin anchor shackle
    pub fn half_inch_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-1/2".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(0.5),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(3250.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 5/8" G-209 screw pin anchor shackle
    pub fn five_eighths_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-5/8".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(0.625),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(4750.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 3/4" G-209 screw pin anchor shackle
    pub fn three_quarter_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-3/4".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(0.75),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(6500.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 7/8" G-209 screw pin anchor shackle
    pub fn seven_eighths_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-7/8".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(0.875),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(8500.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 1" G-209 screw pin anchor shackle
    pub fn one_inch_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-1".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(1.0),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(9500.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 1-1/8" G-209 screw pin anchor shackle
    pub fn one_eighth_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-1-1/8".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(1.125),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(12000.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 1-1/4" G-209 screw pin anchor shackle
    pub fn one_quarter_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-1-1/4".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(1.25),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(13500.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 1-1/2" G-209 screw pin anchor shackle
    pub fn one_half_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-1-1/2".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(1.5),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(17000.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// 2" G-209 screw pin anchor shackle
    pub fn two_inch_anchor_screw() -> Hardware {
        Hardware {
            id: "G-209-2".into(),
            hardware_type: HardwareType::Shackle(ShackleType::Anchor {
                size: Distance::new::<inch>(2.0),
                pin_type: ShacklePinType::ScrewPin,
            }),
            rated_capacity: Weight::new::<pound>(25000.0),
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
}

/// Standard master link specifications
pub mod master_links {
    use super::*;
    
    /// Crosby S-5287 forged master link - 4 attachment points
    pub fn crosby_s5287(size: Distance, capacity: Weight) -> Hardware {
        Hardware {
            id: format!("S-5287-{}", size.get::<inch>()),
            hardware_type: HardwareType::MasterLink(MasterLinkType::Forged {
                num_attachments: 4,
            }),
            rated_capacity: capacity,
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// Common master link sizes
    pub fn half_ton() -> Hardware {
        crosby_s5287(Distance::new::<inch>(0.375), Weight::new::<pound>(1000.0))
    }
    
    pub fn one_ton() -> Hardware {
        crosby_s5287(Distance::new::<inch>(0.5), Weight::new::<pound>(2000.0))
    }
    
    pub fn two_ton() -> Hardware {
        crosby_s5287(Distance::new::<inch>(0.75), Weight::new::<pound>(4000.0))
    }
    
    pub fn three_ton() -> Hardware {
        crosby_s5287(Distance::new::<inch>(1.0), Weight::new::<pound>(6000.0))
    }
}

/// Standard hook specifications
pub mod hooks {
    use super::*;
    
    /// Crosby G-319 eye hook with latch
    pub fn crosby_g319(throat: Distance, capacity: Weight) -> Hardware {
        Hardware {
            id: format!("G-319-{}", capacity.get::<ton>()),
            hardware_type: HardwareType::Hook(HookType::Eye {
                throat_opening: throat,
                has_latch: true,
            }),
            rated_capacity: capacity,
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Crosby".into(),
        }
    }
    
    /// Common hook sizes
    pub fn quarter_ton() -> Hardware {
        crosby_g319(Distance::new::<inch>(0.625), Weight::new::<pound>(500.0))
    }
    
    pub fn half_ton() -> Hardware {
        crosby_g319(Distance::new::<inch>(0.875), Weight::new::<pound>(1000.0))
    }
    
    pub fn one_ton() -> Hardware {
        crosby_g319(Distance::new::<inch>(1.0), Weight::new::<pound>(2000.0))
    }
    
    pub fn two_ton() -> Hardware {
        crosby_g319(Distance::new::<inch>(1.5), Weight::new::<pound>(4000.0))
    }
    
    pub fn three_ton() -> Hardware {
        crosby_g319(Distance::new::<inch>(1.75), Weight::new::<pound>(6000.0))
    }
}

/// Eye bolt specifications
pub mod eye_bolts {
    use super::*;
    
    /// Regular eye bolt - VERTICAL LOAD ONLY
    pub fn regular_eye_bolt(thread_size: Distance, capacity: Weight) -> Hardware {
        Hardware {
            id: format!("EYE-REG-{}", thread_size.get::<inch>()),
            hardware_type: HardwareType::EyeBolt(EyeBoltType::Regular {
                thread_diameter: thread_size,
            }),
            rated_capacity: capacity,
            material: HardwareMaterial::CarbonSteel,
            manufacturer: "Generic".into(),
        }
    }
    
    /// Shoulder eye bolt - can handle angular loads
    pub fn shoulder_eye_bolt(thread_size: Distance, capacity: Weight) -> Hardware {
        Hardware {
            id: format!("EYE-SHOULDER-{}", thread_size.get::<inch>()),
            hardware_type: HardwareType::EyeBolt(EyeBoltType::Shoulder {
                thread_diameter: thread_size,
            }),
            rated_capacity: capacity,
            material: HardwareMaterial::AlloySteel { grade: SteelGrade::Grade80 },
            manufacturer: "Generic".into(),
        }
    }
    
    /// Common regular eye bolt sizes
    pub fn quarter_inch_regular() -> Hardware {
        regular_eye_bolt(Distance::new::<inch>(0.25), Weight::new::<pound>(350.0))
    }
    
    pub fn three_eighths_regular() -> Hardware {
        regular_eye_bolt(Distance::new::<inch>(0.375), Weight::new::<pound>(800.0))
    }
    
    pub fn half_inch_regular() -> Hardware {
        regular_eye_bolt(Distance::new::<inch>(0.5), Weight::new::<pound>(1200.0))
    }
    
    pub fn five_eighths_regular() -> Hardware {
        regular_eye_bolt(Distance::new::<inch>(0.625), Weight::new::<pound>(2000.0))
    }
    
    pub fn three_quarter_regular() -> Hardware {
        regular_eye_bolt(Distance::new::<inch>(0.75), Weight::new::<pound>(3000.0))
    }
    
    /// Common shoulder eye bolt sizes
    pub fn half_inch_shoulder() -> Hardware {
        shoulder_eye_bolt(Distance::new::<inch>(0.5), Weight::new::<pound>(1800.0))
    }
    
    pub fn five_eighths_shoulder() -> Hardware {
        shoulder_eye_bolt(Distance::new::<inch>(0.625), Weight::new::<pound>(3000.0))
    }
    
    pub fn three_quarter_shoulder() -> Hardware {
        shoulder_eye_bolt(Distance::new::<inch>(0.75), Weight::new::<pound>(4500.0))
    }
    
    pub fn one_inch_shoulder() -> Hardware {
        shoulder_eye_bolt(Distance::new::<inch>(1.0), Weight::new::<pound>(7200.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_shackle_inline_capacity() {
        let shackle = crosby_shackles::half_inch_anchor_screw();
        
        let capacity = shackle.effective_capacity(LoadingCondition::InLine);
        
        assert_relative_eq!(capacity.get::<pound>(), 3250.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_shackle_side_load_reduction() {
        let shackle = crosby_shackles::half_inch_anchor_screw();
        
        // 45 degree side load
        let capacity = shackle.effective_capacity(LoadingCondition::SideLoad {
            angle: Angle::new::<degree>(45.0),
        });
        
        // Should be 70% of rated capacity
        assert_relative_eq!(
            capacity.get::<pound>(),
            3250.0 * 0.7,
            epsilon = 1.0
        );
    }
    
    #[test]
    fn test_regular_eye_bolt_angular_unsafe() {
        let eye_bolt = eye_bolts::half_inch_regular();
        
        // Regular eye bolt at angle - UNSAFE!
        let capacity = eye_bolt.effective_capacity(LoadingCondition::Angular {
            angle: Angle::new::<degree>(30.0),
        });
        
        // Should be ZERO - regular eye bolts can't handle angular loads
        assert_relative_eq!(capacity.get::<pound>(), 0.0);
    }
    
    #[test]
    fn test_shoulder_eye_bolt_angular() {
        let eye_bolt = eye_bolts::half_inch_shoulder();
        
        // Shoulder eye bolt at 45 degrees
        let capacity = eye_bolt.effective_capacity(LoadingCondition::Angular {
            angle: Angle::new::<degree>(45.0),
        });
        
        // Should be 70% of rated capacity
        assert_relative_eq!(
            capacity.get::<pound>(),
            1800.0 * 0.7,
            epsilon = 1.0
        );
    }
    
    #[test]
    fn test_hardware_safety_check() {
        let shackle = crosby_shackles::three_quarter_anchor_screw();
        
        // Safe load in-line
        assert!(shackle.is_safe(
            Weight::new::<pound>(5000.0),
            LoadingCondition::InLine
        ));
        
        // Overload
        assert!(!shackle.is_safe(
            Weight::new::<pound>(10000.0),
            LoadingCondition::InLine
        ));
        
        // Safe with side load reduction
        assert!(shackle.is_safe(
            Weight::new::<pound>(3000.0),
            LoadingCondition::SideLoad {
                angle: Angle::new::<degree>(45.0),
            }
        ));
    }
}