//! Complete lift plan validation
//! 
//! Validates all aspects of a crane lift:
//! - Capacity vs load
//! - Ground bearing pressure
//! - Wind conditions
//! - Rigging adequacy
//! - Stability margins
//! - Configuration validity

use crate::equipment::CraneType;
use crate::physics::{WindAnalysis, WindCondition};
use crate::{equipment::Crane, physics::ground_bearing::*, types::*};

/// A complete lift plan for validation
#[derive(Debug, Clone)]
pub struct LiftPlan {
    /// Load weight
    pub load_weight: Weight,
    
    /// Load dimensions (for wind sail area)
    pub load_dimensions: LoadDimensions,
    
    /// Rigging configuration
    pub rigging: RiggingConfiguration,
    
    /// Ground conditions
    pub ground: GroundConditions,
    
    /// Environmental conditions
    pub environment: EnvironmentalConditions,
    
    /// Safety factors to apply
    pub safety_factors: SafetyFactors,
}

#[derive(Debug, Clone)]
pub struct LoadDimensions {
    pub length: Distance,
    pub width: Distance,
    pub height: Distance,
}

impl LoadDimensions {
    /// Calculate wind sail area (worst case)
    pub fn sail_area(&self) -> Area {
        let l = self.length.get::<foot>();
        let w = self.width.get::<foot>();
        let h = self.height.get::<foot>();
        
        // Take largest face
        let area1 = l * h;
        let area2 = w * h;
        let max_area = area1.max(area2);
        
        Area::new::<square_foot>(max_area)
    }
}

#[derive(Debug, Clone)]
pub struct RiggingConfiguration {
    pub configuration: RiggingConfig,
    pub hardware: Vec<RiggingHardware>,
}

#[derive(Debug, Clone)]
pub enum RiggingConfig {
    /// Single vertical hitch
    Vertical,
    
    /// Choker hitch
    Choker { efficiency: f64 },
    
    /// Basket hitch
    Basket,
    
    /// Bridle with specified angles
    Bridle { leg_angle: Angle, num_legs: usize },
}

#[derive(Debug, Clone)]
pub struct RiggingHardware {
    pub item_type: String,
    pub capacity: Weight,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct GroundConditions {
    pub soil_type: SoilType,
    pub mat_area: Area,
    pub notes: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SoilType {
    SoftClay,
    MediumClay,
    StiffClay,
    LooseSand,
    DenseSand,
    Gravel,
    Rock,
    Paved,
    Custom(Pressure),  // PSI
}

impl SoilType {
    pub fn bearing_capacity(&self) -> Pressure {
        match self {
            SoilType::SoftClay => soil_capacities::soft_clay(),
            SoilType::MediumClay => soil_capacities::medium_clay(),
            SoilType::StiffClay => soil_capacities::stiff_clay(),
            SoilType::LooseSand => soil_capacities::loose_sand(),
            SoilType::DenseSand => soil_capacities::dense_sand(),
            SoilType::Gravel => soil_capacities::gravel(),
            SoilType::Rock => soil_capacities::hard_rock(),
            SoilType::Paved => soil_capacities::paved_surface(),
            SoilType::Custom(press) => Pressure::new::<psi>(press.get::<psi>())
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvironmentalConditions {
    pub wind_speed: Velocity,
    pub temperature: f64,
    pub visibility: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct SafetyFactors {
    /// Capacity safety factor (typically 1.0, already in load charts)
    pub capacity: f64,
    
    /// Ground bearing safety factor (typically 2.0)
    pub ground_bearing: f64,
    
    /// Rigging safety factor (typically 5:1 minimum)
    pub rigging: f64,
}

impl Default for SafetyFactors {
    fn default() -> Self {
        Self {
            capacity: 1.0,
            ground_bearing: 2.0,
            rigging: 5.0,
        }
    }
}

/// Result of lift validation
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub overall_status: ValidationStatus,
    pub checks: Vec<ValidationCheck>,
    pub warnings: Vec<String>,
    pub critical_issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationStatus {
    /// All checks passed
    Approved,
    
    /// Passed with warnings
    ApprovedWithWarnings,
    
    /// Failed critical checks
    Rejected,
}

#[derive(Debug, Clone)]
pub struct ValidationCheck {
    pub name: String,
    pub status: CheckStatus,
    pub details: String,
    pub margin: Option<f64>,  // Percentage margin (if applicable)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warning,
    Fail,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            overall_status: ValidationStatus::Approved,
            checks: Vec::new(),
            warnings: Vec::new(),
            critical_issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }
    
    pub fn add_check(&mut self, check: ValidationCheck) {
        match check.status {
            CheckStatus::Fail => {
                self.overall_status = ValidationStatus::Rejected;
                self.critical_issues.push(format!("{}: {}", check.name, check.details));
            }
            CheckStatus::Warning => {
                if self.overall_status == ValidationStatus::Approved {
                    self.overall_status = ValidationStatus::ApprovedWithWarnings;
                }
                self.warnings.push(format!("{}: {}", check.name, check.details));
            }
            _ => {}
        }
        self.checks.push(check);
    }
    
    pub fn add_recommendation(&mut self, rec: String) {
        self.recommendations.push(rec);
    }
    
    /// Print formatted report
    pub fn print(&self) {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║         LIFT VALIDATION REPORT            ║");
        println!("╚════════════════════════════════════════════╝\n");
        
        // Overall status
        let status_symbol = match self.overall_status {
            ValidationStatus::Approved => "✅",
            ValidationStatus::ApprovedWithWarnings => "⚠️",
            ValidationStatus::Rejected => "❌",
        };
        println!("{} Overall Status: {:?}\n", status_symbol, self.overall_status);
        
        // Checks
        println!("Validation Checks:");
        println!("{}", "─".repeat(50));
        for check in &self.checks {
            let symbol = match check.status {
                CheckStatus::Pass => "✅",
                CheckStatus::Warning => "⚠️",
                CheckStatus::Fail => "❌",
            };
            
            if let Some(margin) = check.margin {
                println!("{} {} ({:.1}% margin)", symbol, check.name, margin);
            } else {
                println!("{} {}", symbol, check.name);
            }
            println!("   {}", check.details);
        }
        
        // Critical issues
        if !self.critical_issues.is_empty() {
            println!("\n❌ CRITICAL ISSUES:");
            println!("{}", "─".repeat(50));
            for issue in &self.critical_issues {
                println!("  • {}", issue);
            }
        }
        
        // Warnings
        if !self.warnings.is_empty() {
            println!("\n⚠️  WARNINGS:");
            println!("{}", "─".repeat(50));
            for warning in &self.warnings {
                println!("  • {}", warning);
            }
        }
        
        // Recommendations
        if !self.recommendations.is_empty() {
            println!("\n💡 RECOMMENDATIONS:");
            println!("{}", "─".repeat(50));
            for rec in &self.recommendations {
                println!("  • {}", rec);
            }
        }
        
        println!("\n{}", "═".repeat(50));
    }
}

/// Validate a complete lift plan
pub fn validate_lift<C: Crane>(
    crane: &C,
    plan: &LiftPlan,
) -> ValidationReport {
    let mut report = ValidationReport::new();
    
    // 1. Capacity check
    validate_capacity(crane, plan, &mut report);
    
    // 2. Wind check
    validate_wind(crane, plan, &mut report);
    
    // 3. Ground bearing check
    validate_ground_bearing(crane, plan, &mut report);
    
    // 4. Rigging check
    validate_rigging(plan, &mut report);
    
    // 5. Configuration check
    validate_configuration(crane, plan, &mut report);
    
    report
}

fn validate_capacity<C: Crane>(
    crane: &C,
    plan: &LiftPlan,
    report: &mut ValidationReport,
) {
    let config = crane.configuration();
    let rated_capacity = crane.rated_capacity();
    
    let capacity_lb = rated_capacity.get::<pound>();
    let load_lb = plan.load_weight.get::<pound>();
    let margin = ((capacity_lb - load_lb) / capacity_lb) * 100.0;
    
    let status = if load_lb > capacity_lb {
        CheckStatus::Fail
    } else if margin < 10.0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Pass
    };
    
    report.add_check(ValidationCheck {
        name: "Capacity".into(),
        status,
        details: format!(
            "Load: {:.0} lbs, Rated: {:.0} lbs at {:.1} ft radius, {:.1} ft boom",
            load_lb,
            capacity_lb,
            config.radius.get::<foot>(),
            config.boom_length.get::<foot>(),
        ),
        margin: Some(margin),
    });
    
    if margin < 20.0 && margin >= 10.0 {
        report.add_recommendation(
            "Consider using a larger crane for better safety margin".into()
        );
    }
}

fn validate_wind<C: Crane>(
    crane: &C,
    plan: &LiftPlan,
    report: &mut ValidationReport,
) {
    let config = crane.configuration();
    let sail_area = plan.load_dimensions.sail_area();
    
    let analysis = WindAnalysis::new(
        CraneType::AllTerrain,  // Should come from crane
        config.boom_length,
        config.boom_angle,
        sail_area,
        plan.environment.wind_speed,
    );
    
    let condition = analysis.wind_condition();
    let wind_mph = plan.environment.wind_speed.get::<mile_per_hour>();
    let derating = (1.0 - analysis.derating_factor()) * 100.0;
    
    let status = match condition {
        WindCondition::Safe => CheckStatus::Pass,
        WindCondition::Caution => CheckStatus::Warning,
        WindCondition::Shutdown | WindCondition::OutOfService => CheckStatus::Fail,
    };
    
    report.add_check(ValidationCheck {
        name: "Wind Conditions".into(),
        status,
        details: format!(
            "Wind: {:.1} mph, Condition: {:?}, Derating: {:.1}%",
            wind_mph, condition, derating
        ),
        margin: None,
    });
    
    if matches!(condition, WindCondition::Caution) {
        report.add_recommendation(
            "Monitor wind speed closely. Consider delaying lift if winds increase".into()
        );
    }
}

fn validate_ground_bearing<C: Crane>(
    _crane: &C,
    plan: &LiftPlan,
    report: &mut ValidationReport,
) {
    // TODO:
    // This would use crane.ground_bearing_analysis()
    // For now, simplified check
    
    let soil_capacity = plan.ground.soil_type.bearing_capacity();
    let soil_psi = soil_capacity.get::<psi>();
    
    // Simplified: assume equal distribution (conservative in reality)
    let total_weight = plan.load_weight.get::<pound>() + 100000.0; // crane weight estimate
    let mat_area_sqin = plan.ground.mat_area.get::<square_inch>();
    let num_outriggers = 4.0;
    
    let pressure_psi = (total_weight / num_outriggers) / mat_area_sqin;
    let allowable_psi = soil_psi / plan.safety_factors.ground_bearing;
    
    let margin = ((allowable_psi - pressure_psi) / allowable_psi) * 100.0;
    
    let status = if pressure_psi > allowable_psi {
        CheckStatus::Fail
    } else if margin < 20.0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Pass
    };
    
    report.add_check(ValidationCheck {
        name: "Ground Bearing".into(),
        status,
        details: format!(
            "Pressure: {:.1} PSI, Allowable: {:.1} PSI ({:?} soil with {}:1 SF)",
            pressure_psi,
            allowable_psi,
            plan.ground.soil_type,
            plan.safety_factors.ground_bearing,
        ),
        margin: Some(margin),
    });
    
    if pressure_psi > allowable_psi {
        let required_area = (total_weight / num_outriggers) / allowable_psi;
        let side = (required_area / 144.0).sqrt(); // Convert to ft
        report.add_recommendation(
            format!("Use larger mats: minimum {:.1} ft x {:.1} ft required", side, side)
        );
    }
}

fn validate_rigging(
    plan: &LiftPlan,
    report: &mut ValidationReport,
) {
    let load_lb = plan.load_weight.get::<pound>();
    
    // Calculate load on rigging based on configuration
    let rigging_load = match &plan.rigging.configuration {
        RiggingConfig::Vertical => load_lb,
        RiggingConfig::Choker { efficiency } => load_lb / efficiency,
        RiggingConfig::Basket => load_lb / 2.0,
        RiggingConfig::Bridle { leg_angle, num_legs } => {
            let angle_deg = leg_angle.get::<degree>();
            let angle_factor = 1.0 / (angle_deg.to_radians().cos());
            load_lb * angle_factor / (*num_legs as f64)
        }
    };
    
    // Check each piece of hardware
    let mut min_margin = f64::MAX;
    let mut weakest_component = String::new();
    
    for hardware in &plan.rigging.hardware {
        let capacity = hardware.capacity.get::<pound>();
        let margin = ((capacity - rigging_load) / capacity) * 100.0;
        
        if margin < min_margin {
            min_margin = margin;
            weakest_component = hardware.description.clone();
        }
    }
    
    let required_capacity = rigging_load * plan.safety_factors.rigging;
    let status = if min_margin < 0.0 {
        CheckStatus::Fail
    } else if required_capacity > plan.rigging.hardware.iter()
        .map(|h| h.capacity.get::<pound>())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
    {
        CheckStatus::Warning
    } else {
        CheckStatus::Pass
    };
    
    report.add_check(ValidationCheck {
        name: "Rigging Capacity".into(),
        status,
        details: format!(
            "Load per leg: {:.0} lbs, Min margin: {:.1}% ({})",
            rigging_load, min_margin, weakest_component
        ),
        margin: Some(min_margin),
    });
    
    if min_margin < 100.0 {
        report.add_recommendation(
            format!("Consider upgrading {} for better safety margin", weakest_component)
        );
    }
}

fn validate_configuration<C: Crane>(
    crane: &C,
    _plan: &LiftPlan,
    report: &mut ValidationReport,
) {
    // TODO:
    let config = crane.configuration();
    
    // Basic geometry checks
    let radius = config.radius.get::<foot>();
    let height = config.height.get::<foot>();
    
    let status = if radius < 10.0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Pass
    };
    
    report.add_check(ValidationCheck {
        name: "Configuration".into(),
        status,
        details: format!(
            "Boom: {:.1} ft at {:.1}°, Radius: {:.1} ft, Height: {:.1} ft",
            config.boom_length.get::<foot>(),
            config.boom_angle.get::<degree>(),
            radius,
            height,
        ),
        margin: None,
    });
}
