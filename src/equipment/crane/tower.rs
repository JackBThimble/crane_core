use nalgebra as na;
use crate::equipment::crane::{Crane, CraneConfig, LiftError};
use crate::types::*;
use crate::kinematics::{ForwardKinematics, JointConfig, CraneBase};

/// Tower crane - fixed base with slewing superstructure
/// 
/// Key differences from mobile cranes:
/// - Rated by MOMENT (ton-meters or foot-pounds), not capacity at radius
/// - Fixed to ground or building structure
/// - Trolley moves load along jib
/// - Huge counterweights on machinery deck
/// - Don't "tip" - limited by moment capacity
#[derive(Debug, Clone)]
pub struct TowerCrane {
    pub manufacturer: String,
    pub model: String,
    
    /// Tower crane configuration type
    pub crane_type: TowerCraneType,
    
    /// Tower height (from base to slewing ring)
    pub tower_height: Length,
    
    /// Jib configuration
    pub jib: TowerJib,
    
    /// Current slew angle (rotation)
    pub slew_angle: Angle,
    
    /// Trolley position (distance from tower center along jib)
    pub trolley_position: Length,
    
    /// Hook height above trolley
    pub hook_height: Length,
    
    /// Counterweight configuration
    pub counterweight: CounterweightConfig,
    
    /// Maximum rated moment (THE critical rating)
    pub max_moment: TowerMoment,
    
    /// Load moment limiter settings
    pub moment_limiter: MomentLimiter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TowerCraneType {
    /// Traditional hammerhead with cat-head and A-frame
    Hammerhead,
    
    /// Flat-top design (no cat-head, easier multi-crane coordination)
    FlatTop,
    
    /// Luffing jib (jib angle adjustable, better for tight sites)
    LuffingJib,
    
    /// Self-erecting (smaller, mobile base)
    SelfErecting,
}

/// Tower crane jib configuration
#[derive(Debug, Clone)]
pub struct TowerJib {
    /// Jib length (tip to tower center)
    pub length: Length,
    
    /// Jib angle from horizontal (for luffing jibs)
    /// Fixed at 0° for hammerhead/flat-top
    pub angle: Angle,
    
    /// Minimum radius (can't get closer to tower than this)
    pub min_radius: Length,
    
    /// Maximum radius (jib tip)
    pub max_radius: Length,
}

/// Counterweight configuration
#[derive(Debug, Clone)]
pub struct CounterweightConfig {
    /// Total counterweight mass
    pub weight: Mass,
    
    /// Length from slewing center to counterweight COG
    pub radius: Length,
    
    /// Counterweight moment (weight × radius)
    pub moment: TowerMoment,
}

impl CounterweightConfig {
    pub fn new(weight: Mass, radius: Length) -> Self {
        let moment = TowerMoment::new(
            weight.get::<pound>() * radius.get::<foot>()
        );
        
        Self {
            weight,
            radius,
            moment,
        }
    }
}


/// Moment rating for tower cranes (load × radius)
/// 
/// This is THE critical rating. Tower cranes are limited by moment, not load.
/// A 10,000 lb load at 100 ft = 1,000,000 ft-lb moment
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TowerMoment(pub f64); // ft-lb

impl TowerMoment {
    #[inline(always)]
    pub fn new(ft_lb: f64) -> Self {
        Self(ft_lb)
    }
    
    #[inline(always)]
    pub fn from_ton_meters(ton_m: f64) -> Self {
        // 1 ton-meter = 6720 ft-lb
        Self(ton_m * 6720.0)
    }
    
    #[inline(always)]
    pub fn ft_lb(self) -> f64 {
        self.0
    }
    
    #[inline(always)]
    pub fn ton_meters(self) -> f64 {
        self.0 / 6720.0
    }
    
    /// Calculate moment from load and radius
    pub fn from_load(load: Mass, radius: Length) -> Self {
        Self(load.get::<pound>() * radius.get::<foot>())
    }
}

use std::fmt;
#[derive(Debug)]
pub struct DisplayTowerMoment(pub TowerMoment);
impl fmt::Display for DisplayTowerMoment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} ft-lb ({:.3} ton-m)", self.0.ft_lb(), self.0.ton_meters())
    }
}

#[derive(Debug, Clone, Copy)] 
pub struct SafetyMargins{
    /// The safety factor applies a safety margin to
    /// the effective moment of the MomentLimiter.
    pub safety_factor: f64,
    /// The warning factor applies a safety margin
    /// to the warning_threshold. 
    /// A warning factor of 0.9 will yield a 
    /// warning_threshold of 90% of the 
    /// effective moment.
    /// Should be <= 1.0
    /// Defaults to 0.9
    pub warning_factor: f64,
    /// The shutdown factor will apply a margin to 
    /// the shutdown_threshold of the MomentLimiter.
    /// Typically 1.0 to 1.05 (100% to 105% of the rated moment)
    pub shutdown_factor: f64,
}

impl Default for SafetyMargins {
    fn default() -> Self {
        Self::standard()
    }
}

impl SafetyMargins {
    /// Standard operations
    /// 90% warning, 100% shutdown
    /// Safety factor of 1.0
    pub const fn standard() -> Self {
        Self {
            warning_factor: 0.9,
            shutdown_factor: 1.0,
            safety_factor: 1.0,
        }
    }
    
    /// Conservative margins
    /// 85% warning, 95% shutdown
    pub const fn conservative() -> Self {
        Self {
            warning_factor: 0.85,
            shutdown_factor: 0.95,
            safety_factor: 1.0,
        }
    }
    
    pub const fn very_conservative() -> Self {
        Self {
            warning_factor: 0.80,
            shutdown_factor: 0.90,
            safety_factor: 1.0,
        }
    }
    
    pub const fn with_safety_factor(factor: f64) -> Self {
        Self {
            warning_factor: 0.9,
            shutdown_factor: 1.0,
            safety_factor: factor
        }
    }
    
    pub const fn custom(warning: f64, shutdown: f64) -> Self {
        Self {
            warning_factor: warning,
            shutdown_factor: shutdown,
            safety_factor: 1.0,
        }
    }
    
     /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.warning_factor <= 0.0 || self.warning_factor > 1.0 {
            return Err(format!("Warning threshold must be 0.0 < x <= 1.0, got {}", self.warning_factor));
        }
        
        if self.shutdown_factor <= 0.0 || self.shutdown_factor > 1.0 {
            return Err(format!("Shutdown threshold must be 0.0 < x <= 1.0, got {}", self.shutdown_factor));
        }
        
        if self.safety_factor <= 0.0 || self.safety_factor > 1.0 {
            return Err(format!("Safety factor must be 0.0 < x <= 1.0, got {}", self.safety_factor));
        }
        
        if self.warning_factor >= self.shutdown_factor {
            return Err(format!(
                "Warning threshold ({}) must be less than shutdown threshold ({})",
                self.warning_factor, self.shutdown_factor
            ));
        }
        
        Ok(())
    }
    
    /// Calculate effective warning moment
    pub fn effective_warning(&self, rated_moment: TowerMoment) -> TowerMoment {
        TowerMoment::new(rated_moment.0 * self.warning_factor * self.safety_factor)
    }
    
    /// Calculate effective shutdown moment
    pub fn effective_shutdown(&self, rated_moment: TowerMoment) -> TowerMoment {
        TowerMoment::new(rated_moment.0 * self.shutdown_factor * self.safety_factor)
    }
}
/// Load moment limiter system
/// 
/// This is the safety device that prevents overload
#[derive(Debug, Clone)]
pub struct MomentLimiter {
    /// Rated moment capacity
    pub rated_moment: TowerMoment,
    /// Safety margins
    pub margins: SafetyMargins,
    /// Whether limiter is active
    pub enabled: bool,
    /// Epsilon for floating point comparisons
    epsilon: f64,
}

impl MomentLimiter {
    pub fn new(rated_moment: TowerMoment, margins: SafetyMargins) -> Result<Self, String> {
        margins.validate()?;
        
        Ok(Self {
            rated_moment,
            margins,
            enabled: true,
            epsilon: 0.01,
        })
    }
    
    /// Create with standard margins
    pub fn standard(rated_moment: TowerMoment) -> Self {
        Self::new(rated_moment, SafetyMargins::standard()).unwrap()
    }
    
    pub fn check(&self, current_moment: TowerMoment) -> LimiterStatus {
        if !self.enabled {
            return LimiterStatus::Disabled;
        }
        
        let shutdown = self.margins.effective_shutdown(self.rated_moment);
        let warning = self.margins.effective_warning(self.rated_moment);
        
        if current_moment.0 > shutdown.0 - self.epsilon {
            LimiterStatus::Shutdown
        } else if current_moment.0 > warning.0 - self.epsilon {
            LimiterStatus::Warning
        } else {
            LimiterStatus::Normal
        }
    }
    
    pub fn effective_capacity(&self) -> TowerMoment {
        self.margins.effective_shutdown(self.rated_moment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimiterStatus {
    Normal,
    Warning,
    Shutdown,
    Disabled,
}

impl TowerCrane {
    pub fn new(
        manufacturer: impl Into<String>,
        model: impl Into<String>,
        crane_type: TowerCraneType,
        tower_height: Length,
        jib_length: Length,
        max_moment: TowerMoment,
    ) -> Self {
        let jib = TowerJib {
            length: jib_length,
            angle: Angle::new::<degree>(0.0),
            min_radius: Length::new::<foot>(20.0), // Typical minimum
            max_radius: jib_length,
        };
        
        Self {
            manufacturer: manufacturer.into(),
            model: model.into(),
            crane_type,
            tower_height,
            jib,
            slew_angle: Angle::new::<degree>(0.0),
            trolley_position: Length::new::<foot>(50.0),
            hook_height: Length::new::<foot>(0.0),
            counterweight: CounterweightConfig::new(
                Mass::new::<pound>(20000.0),
                Length::new::<foot>(20.0),
            ),
            max_moment,
            moment_limiter: MomentLimiter::new(max_moment, SafetyMargins::standard()).unwrap(),
        }
    }
    
    /// Calculate current load moment
    /// 
    /// This is THE critical calculation for tower cranes
    pub fn load_moment(&self, load: Mass) -> TowerMoment {
        TowerMoment::from_load(load, self.trolley_position)
    }
    
    /// Calculate available capacity at current trolley position
    /// 
    /// Capacity = Max Moment / Current Radius
    pub fn capacity_at_current_position(&self) -> Mass {
        let radius = self.trolley_position.get::<foot>();
        
        if radius < 0.01 {
            // Can't lift at zero radius (physically impossible)
            return Mass::new::<pound>(0.0);
        }
        
        let capacity_lb = self.max_moment.0 / radius;
        Mass::new::<pound>(capacity_lb)
    }
    
    /// Calculate capacity at any trolley position
    pub fn capacity_at_radius(&self, radius: Length) -> Mass {
        let r = radius.get::<foot>();
        
        if r < self.jib.min_radius.get::<foot>() {
            return Mass::new::<pound>(0.0);
        }
        
        if r > self.jib.max_radius.get::<foot>() {
            return Mass::new::<pound>(0.0);
        }
        
        let capacity_lb = self.max_moment.0 / r;
        Mass::new::<pound>(capacity_lb)
    }
    
    /// Set trolley position (with range checking)
    pub fn set_trolley_position(&mut self, radius: Length) -> Result<(), TowerCraneError> {
        if radius < self.jib.min_radius {
            return Err(TowerCraneError::RadiusTooSmall {
                requested: DisplayLength(radius),
                minimum: DisplayLength(self.jib.min_radius),
            });
        }
        
        if radius > self.jib.max_radius {
            return Err(TowerCraneError::RadiusTooLarge {
                requested: DisplayLength(radius),
                maximum: DisplayLength(self.jib.max_radius),
            });
        }
        
        self.trolley_position = radius;
        Ok(())
    }
    
    /// Calculate hook position in 3D space
    pub fn hook_position(&self) -> na::Point3<f64> {
        let tower_height = self.tower_height.get::<foot>();
        let jib_angle = self.jib.angle.get::<radian>();
        let trolley_radius = self.trolley_position.get::<foot>();
        let slew = self.slew_angle.get::<radian>();
        let hook_drop = self.hook_height.get::<foot>();
        
        // Tower cranes: trolley moves along jib
        // For hammerhead/flat-top, jib is horizontal (angle = 0)
        let jib_height = tower_height + trolley_radius * jib_angle.sin();
        let jib_horizontal = trolley_radius * jib_angle.cos();
        
        // Hook hangs below trolley
        let hook_height = jib_height - hook_drop;
        
        // Apply slew rotation (Y-up, Z-forward)
        na::Point3::new(
            jib_horizontal * slew.sin(),
            hook_height,
            jib_horizontal * slew.cos(),
        )
    }
    
    /// Check moment limiter for given load
    pub fn check_moment_limiter(&self, load: Mass) -> LimiterStatus {
        let moment = self.load_moment(load);
        self.moment_limiter.check(moment)
    }
    
    /// Validate if lift is safe at current configuration
    pub fn validate_lift(&self, load: Mass) -> Result<TowerLiftAnalysis, TowerCraneError> {
        // Check moment capacity
        let load_moment = self.load_moment(load);
        
        if load_moment > self.max_moment {
            return Err(TowerCraneError::MomentExceeded {
                load_moment: DisplayTowerMoment(load_moment),
                max_moment: DisplayTowerMoment(self.max_moment),
            });
        }
        
        // Check trolley position
        if self.trolley_position < self.jib.min_radius {
            return Err(TowerCraneError::RadiusTooSmall {
                requested: DisplayLength(self.trolley_position),
                minimum: DisplayLength(self.jib.min_radius),
            });
        }
        
        if self.trolley_position > self.jib.max_radius {
            return Err(TowerCraneError::RadiusTooLarge {
                requested: DisplayLength(self.trolley_position),
                maximum: DisplayLength(self.jib.max_radius),
            });
        }
        
        // Check moment limiter
        let limiter_status = self.check_moment_limiter(load);
        
        if limiter_status == LimiterStatus::Shutdown {
            return Err(TowerCraneError::MomentLimiterShutdown {
                current_moment: DisplayTowerMoment(load_moment),
            });
        }
        
        // Calculate utilization
        let capacity = self.capacity_at_current_position();
        let utilization = load.get::<pound>() / capacity.get::<pound>();
        
        Ok(TowerLiftAnalysis {
            load,
            radius: self.trolley_position,
            load_moment,
            max_moment: self.max_moment,
            capacity,
            utilization,
            limiter_status,
            is_safe: limiter_status != LimiterStatus::Shutdown,
        })
    }
}

/// Tower crane lift analysis results
#[derive(Debug)]
pub struct TowerLiftAnalysis {
    pub load: Mass,
    pub radius: Length,
    pub load_moment: TowerMoment,
    pub max_moment: TowerMoment,
    pub capacity: Mass,
    pub utilization: f64,
    pub limiter_status: LimiterStatus,
    pub is_safe: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum TowerCraneError {
    #[error("Load moment {load_moment} exceeds maximum moment {max_moment}")]
    MomentExceeded {
        load_moment: DisplayTowerMoment,
        max_moment: DisplayTowerMoment,
    },
    
    #[error("Radius {requested} is less than minimum {minimum}")]
    RadiusTooSmall {
        requested: DisplayLength,
        minimum: DisplayLength,
    },
    
    #[error("Radius {requested} exceeds maximum {maximum}")]
    RadiusTooLarge {
        requested: DisplayLength,
        maximum: DisplayLength,
    },
    
    #[error("Moment limiter shutdown: current moment {current_moment}")]
    MomentLimiterShutdown {
        current_moment: DisplayTowerMoment,
    },
}

// Implement Crane trait for TowerCrane
impl Crane for TowerCrane {
    fn configuration(&self) -> CraneConfig {
        CraneConfig {
            boom_length: self.jib.length,
            boom_angle: self.jib.angle,
            radius: self.trolley_position,
            height: self.tower_height,
        }
    }
    
    fn tip_position(&self) -> na::Point3<f64> {
        self.hook_position()
    }
    
    fn load_chart(&self) -> &crate::capacity::load_chart::LoadChart {
        // Tower cranes don't use traditional load charts
        // They use moment ratings
        // Return a dummy for now
        todo!("Tower cranes use moment ratings, not load charts")
    }
    
    fn system_cog(&self, load: Mass) -> na::Point3<f64> {
        // Calculate system COG including load and counterweight
        let hook = self.hook_position();
        let cw_radius = self.counterweight.radius.get::<foot>();
        let slew = self.slew_angle.get::<radian>();
        
        // Counterweight is opposite side from load
        let cw_pos = na::Point3::new(
            -cw_radius * slew.sin(),
            self.tower_height.get::<foot>(),
            -cw_radius * slew.cos(),
        );
        
        let total_weight = load.get::<pound>() + self.counterweight.weight.get::<pound>();
        
        let weighted_pos = (hook.coords * load.get::<pound>() + 
                           cw_pos.coords * self.counterweight.weight.get::<pound>()) / total_weight;
        
        na::Point3::from(weighted_pos)
    }
    
    fn tipping_moment(&self, load: Mass) -> f64 {
        // Tower cranes don't "tip" in the traditional sense
        // They're rated by moment capacity
        self.load_moment(load).0
    }
    
    fn rated_capacity(&self) -> Mass {
        self.capacity_at_current_position()
    }
    
    fn validate_lift(&self, load: Mass) -> Result<(), LiftError> {
        match self.validate_lift(load) {
            Ok(_) => Ok(()),
            Err(TowerCraneError::MomentExceeded { load_moment, max_moment }) => {
                Err(LiftError::OverCapacity {
                    load,
                    capacity: Mass::new::<pound>(max_moment.0.ft_lb() / self.trolley_position.get::<foot>()),
                })
            }
            Err(_) => Err(LiftError::LoadChartExceeded {
                radius: self.trolley_position,
            }),
        }
    }
    
    fn forward_kinematics(&self) -> ForwardKinematics {
        let base = CraneBase {
            position: na::Point3::origin(),
            pivot_height: Length::new::<foot>(0.0),
        };
        ForwardKinematics::new(base)
    }
    
    fn joint_config(&self) -> JointConfig {
        JointConfig {
            swing: self.slew_angle,
            boom_angle: self.jib.angle,
            boom_length: self.jib.length,
            jib: None,
        }
    }
    
    fn set_joint_config(&mut self, joints: JointConfig) {
        self.slew_angle = joints.swing;
        self.jib.angle = joints.boom_angle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_moment_calculation() {
        let crane = TowerCrane::new(
            "Liebherr",
            "280 EC-H 12",
            TowerCraneType::FlatTop,
            Length::new::<foot>(200.0),
            Length::new::<foot>(200.0),
            TowerMoment::new(1_000_000.0), // 1M ft-lb moment capacity
        );
        
        // 10,000 lb load at 100 ft = 1,000,000 ft-lb moment
        let load = Mass::new::<pound>(10000.0);
        let mut test_crane = crane.clone();
        test_crane.trolley_position = Length::new::<foot>(100.0);
        
        let moment = test_crane.load_moment(load);
        
        assert_relative_eq!(moment.ft_lb(), 1_000_000.0);
    }
    
    #[test]
    fn test_capacity_at_radius() {
        let crane = TowerCrane::new(
            "Liebherr",
            "280 EC-H 12",
            TowerCraneType::FlatTop,
            Length::new::<foot>(200.0),
            Length::new::<foot>(200.0),
            TowerMoment::new(1_000_000.0),
        );
        
        // At 50 ft: capacity = 1,000,000 / 50 = 20,000 lbs
        let capacity = crane.capacity_at_radius(Length::new::<foot>(50.0));
        
        assert_relative_eq!(capacity.get::<pound>(), 20000.0);
        
        // At 100 ft: capacity = 1,000,000 / 100 = 10,000 lbs
        let capacity = crane.capacity_at_radius(Length::new::<foot>(100.0));
        
        assert_relative_eq!(capacity.get::<pound>(), 10000.0);
    }
    
    #[test]
    fn test_moment_limiter() {
        let crane = TowerCrane::new(
            "Liebherr",
            "280 EC-H 12",
            TowerCraneType::FlatTop,
            Length::new::<foot>(200.0),
            Length::new::<foot>(200.0),
            TowerMoment::new(1_000_000.0),
        );
        
        let mut test_crane = crane.clone();
        test_crane.trolley_position = Length::new::<foot>(100.0);
        
        // Safe load (9,000 lbs at 100 ft = 900,000 ft-lb)
        let safe_status = test_crane.check_moment_limiter(Mass::new::<pound>(9000.0));
        assert_eq!(safe_status, LimiterStatus::Warning);
        
        // Overload (11,000 lbs at 100 ft = 1,100,000 ft-lb)
        let overload_status = test_crane.check_moment_limiter(Mass::new::<pound>(11000.0));
        assert_eq!(overload_status, LimiterStatus::Shutdown);
    }
    
    #[test]
    fn test_validate_lift() {
        let crane = TowerCrane::new(
            "Liebherr",
            "280 EC-H 12",
            TowerCraneType::FlatTop,
            Length::new::<foot>(200.0),
            Length::new::<foot>(200.0),
            TowerMoment::new(1_000_000.0),
        );
        
        let mut test_crane = crane.clone();
        test_crane.trolley_position = Length::new::<foot>(100.0);
        
        // Safe lift
        let analysis = test_crane.validate_lift(Mass::new::<pound>(8000.0)).unwrap();
        assert!(analysis.is_safe);
        assert_relative_eq!(analysis.utilization, 0.8, epsilon = 0.01);
        
        // Overload
        let result = test_crane.validate_lift(Mass::new::<pound>(12000.0));
        assert!(result.is_err());
    }
}
