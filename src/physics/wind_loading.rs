use crate::types::*;
use crate::equipment::CraneType;

#[derive(Debug, Clone)]
pub struct WindAnalysis {
    pub crane_type: CraneType,
    pub boom_length: Length,
    pub boom_angle: Angle,
    pub load_area: Area,
    pub wind_speed: Velocity,
}

#[derive(Debug, thiserror::Error)]
pub enum WindError {
    #[error("Wind speed {actual} exceeds operating limit {limit}")]
    ExceedsOperatingLimit {actual: DisplayVelocity, limit: DisplayVelocity},

    #[error("Wind speed {actual} exceeds shutdown limit {limit} - cease operations immediately")]
    ShutdownRequired {actual: DisplayVelocity, limit: DisplayVelocity},

    #[error("Wind speed {actual} exceeds out-of-service limit - crane damage risk")]
    OutOfServiceExceeded {actual: DisplayVelocity, limit: DisplayVelocity},
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindCondition {
    Safe,
    Caution,
    Shutdown,
    OutOfService,
}

impl WindAnalysis {
    pub fn new(
        crane_type: CraneType,
        boom_length: Length,
        boom_angle: Angle,
        load_area: Area,
        wind_speed: Velocity,
    ) -> Self {
        Self {
            crane_type,
            boom_length,
            boom_angle,
            load_area,
            wind_speed,
        }
    }

    /// Calculate wind derating factor (multiply capacity by this)
    /// 
    /// Returns a factor between 0.0 and 1.0
    /// - 1.0 = no derating (calm winds)
    /// - 0.85 = 15% capacity reduction
    /// - 0.0 = shutdown (winds too high)
    pub fn derating_factor(&self) -> f64 {
        let wind_mph = self.wind_speed.get::<mile_per_hour>();
        
        // Operating limits vary by crane type
        let (caution_wind, shutdown_wind) = self.operating_limits();
        let caution_mph = caution_wind.get::<mile_per_hour>();
        let shutdown_mph = shutdown_wind.get::<mile_per_hour>();
        
        if wind_mph <= caution_mph {
            // Below caution: minimal derating (0-5%)
            let factor = 1.0 - (wind_mph / caution_mph) * 0.05;
            factor.max(0.95)
        } else if wind_mph <= shutdown_mph {
            // Caution range: linear derating from 95% to 0%
            let range = shutdown_mph - caution_mph;
            let position = wind_mph - caution_mph;
            let factor = 0.95 * (1.0 - position / range);
            factor.max(0.0)
        } else {
            // Above shutdown: zero capacity
            0.0
        }
    }
    
    /// Get derated capacity
    pub fn derated_capacity(&self, rated_capacity: Mass) -> Mass {
        let factor = self.derating_factor();
        Mass::new::<pound>(rated_capacity.get::<pound>() * factor)
    }
    
    /// Operating wind speed limits for this crane
    /// 
    /// Returns (caution_threshold, shutdown_threshold)
    pub fn operating_limits(&self) -> (Velocity, Velocity) {
        match self.crane_type {
            CraneType::MobileTelescopic | CraneType::AllTerrain => {
                // Mobile cranes: more wind sensitive
                (
                    Velocity::new::<mile_per_hour>(20.0),  // Caution
                    Velocity::new::<mile_per_hour>(30.0),  // Shutdown
                )
            }
            CraneType::MobileLattice => {
                // Lattice boom: more wind resistance
                (
                    Velocity::new::<mile_per_hour>(25.0),
                    Velocity::new::<mile_per_hour>(35.0),
                )
            }
            CraneType::RoughTerrain => {
                // Similar to mobile telescopic
                (
                    Velocity::new::<mile_per_hour>(20.0),
                    Velocity::new::<mile_per_hour>(30.0),
                )
            }
            CraneType::Crawler => {
                // More stable, higher limits
                (
                    Velocity::new::<mile_per_hour>(25.0),
                    Velocity::new::<mile_per_hour>(35.0),
                )
            }
            CraneType::Tower => {
                // Tower cranes: very wind sensitive
                (
                    Velocity::new::<mile_per_hour>(15.0),
                    Velocity::new::<mile_per_hour>(25.0),
                )
            }
            CraneType::TruckMounted => {
                (
                    Velocity::new::<mile_per_hour>(20.0), 
                    Velocity::new::<mile_per_hour>(30.0),
                )
            }
        }
    }
    
    /// Out-of-service wind speed limit (when unattended)
    /// 
    /// Above this speed, crane must be in storm configuration or risk damage
    pub fn out_of_service_limit(&self) -> Velocity {
        match self.crane_type {
            CraneType::MobileTelescopic | CraneType::AllTerrain | CraneType::TruckMounted => {
                // With boom down: ~70 mph
                // With boom up: ~45 mph (check manufacturer)
                if self.boom_angle.get::<degree>() > 45.0 {
                    Velocity::new::<mile_per_hour>(45.0)
                } else {
                    Velocity::new::<mile_per_hour>(70.0)
                }
            }
            CraneType::MobileLattice => {
                Velocity::new::<mile_per_hour>(60.0)
            }
            CraneType::RoughTerrain => {
                if self.boom_angle.get::<degree>() > 45.0 {
                    Velocity::new::<mile_per_hour>(45.0)
                } else {
                    Velocity::new::<mile_per_hour>(65.0)
                }
            }
            CraneType::Crawler => {
                Velocity::new::<mile_per_hour>(65.0)
            }
            CraneType::Tower => {
                // Must be in weathervane mode
                Velocity::new::<mile_per_hour>(80.0)
            }
        }
    }
    
    /// Classify current wind condition
    pub fn wind_condition(&self) -> WindCondition {
        let wind_mph = self.wind_speed.get::<mile_per_hour>();
        let (caution, shutdown) = self.operating_limits();
        let out_of_service = self.out_of_service_limit();
        
        if wind_mph >= out_of_service.get::<mile_per_hour>() {
            WindCondition::OutOfService
        } else if wind_mph >= shutdown.get::<mile_per_hour>() {
            WindCondition::Shutdown
        } else if wind_mph >= caution.get::<mile_per_hour>() {
            WindCondition::Caution
        } else {
            WindCondition::Safe
        }
    }
    
    /// Validate wind conditions for operation
    pub fn validate_for_operation(&self) -> Result<(), WindError> {
        let condition = self.wind_condition();
        
        match condition {
            WindCondition::Safe => Ok(()),
            WindCondition::Caution => {
                // Allow but warn
                Ok(())
            }
            WindCondition::Shutdown => {
                let (_, shutdown) = self.operating_limits();
                Err(WindError::ShutdownRequired {
                    actual: DisplayVelocity(self.wind_speed),
                    limit: DisplayVelocity(shutdown),
                })
            }
            WindCondition::OutOfService => {
                let limit = self.out_of_service_limit();
                Err(WindError::OutOfServiceExceeded {
                    actual: DisplayVelocity(self.wind_speed),
                    limit: DisplayVelocity(limit),
                })
            }
        }
    }
    
    /// Calculate wind force on boom structure
    /// 
    /// Uses simplified drag equation: F = 0.5 * ρ * v² * Cd * A
    /// Where:
    /// - ρ = air density (~0.00237 slug/ft³)
    /// - v = wind velocity
    /// - Cd = drag coefficient (~1.2 for lattice, ~0.8 for telescopic)
    /// - A = projected area
    pub fn wind_force_on_boom(&self) -> Force {
        let wind_fps = self.wind_speed.get::<foot_per_second>();
        let boom_len_ft = self.boom_length.get::<foot>();
        let angle_rad = self.boom_angle.get::<radian>();
        
        // Air density (slug/ft³)
        let rho = 0.00237;
        
        // Drag coefficient
        let cd = match self.crane_type {
            CraneType::MobileLattice | CraneType::Crawler => 1.2,
            _ => 0.8,
        };
        
        // Projected area (boom diameter * length * sin(angle))
        // Assume typical boom diameter of 3 ft for mobile, 5 ft for lattice
        let boom_diameter = match self.crane_type {
            CraneType::MobileLattice | CraneType::Crawler => 5.0,
            _ => 3.0,
        };
        
        let projected_area = boom_diameter * boom_len_ft * angle_rad.sin().abs();
        
        // Drag equation
        let force_lbf = 0.5 * rho * wind_fps.powi(2) * cd * projected_area;
        
        Force::new::<pound_force>(force_lbf)
    }
    
    /// Calculate wind force on suspended load
    pub fn wind_force_on_load(&self) -> Force {
        let wind_fps = self.wind_speed.get::<foot_per_second>();
        let area_sqft = self.load_area.get::<square_foot>();
        
        let rho = 0.00237;  // Air density
        let cd = 1.5;  // Drag coefficient for bluff body
        
        let force_lbf = 0.5 * rho * wind_fps.powi(2) * cd * area_sqft;
        
        Force::new::<pound_force>(force_lbf)
    }
    
    /// Calculate additional overturning moment due to wind
    /// 
    /// This is the moment at the crane base due to wind forces
    pub fn wind_overturning_moment(&self) -> f64 {
        let boom_force = self.wind_force_on_boom();
        let load_force = self.wind_force_on_load();
        
        let boom_len_ft = self.boom_length.get::<foot>();
        let angle_rad = self.boom_angle.get::<radian>();
        
        // Boom force acts at center of boom
        let boom_moment_arm = (boom_len_ft / 2.0) * angle_rad.cos();
        let boom_moment = boom_force.get::<pound_force>() * boom_moment_arm;
        
        // Load force acts at boom tip
        let load_moment_arm = boom_len_ft * angle_rad.cos();
        let load_moment = load_force.get::<pound_force>() * load_moment_arm;
        
        boom_moment + load_moment
    }
    
    /// Get wind condition summary
    pub fn summary(&self) -> String {
        let wind_mph = self.wind_speed.get::<mile_per_hour>();
        let condition = self.wind_condition();
        let derating = (1.0 - self.derating_factor()) * 100.0;
        let (caution, shutdown) = self.operating_limits();
        
        let mut s = String::new();
        s.push_str("Wind Loading Analysis:\n");
        s.push_str(&format!("\nCurrent wind speed: {:.1} mph\n", wind_mph));
        s.push_str(&format!("Condition: {:?}\n", condition));
        s.push_str(&format!("Capacity derating: {:.1}%\n", derating));
        s.push_str(&format!("\nOperating limits:\n"));
        s.push_str(&format!("  Caution: {:.0} mph\n", caution.get::<mile_per_hour>()));
        s.push_str(&format!("  Shutdown: {:.0} mph\n", shutdown.get::<mile_per_hour>()));
        s.push_str(&format!("  Out-of-service: {:.0} mph\n", 
            self.out_of_service_limit().get::<mile_per_hour>()));
        
        let boom_force = self.wind_force_on_boom();
        let load_force = self.wind_force_on_load();
        s.push_str(&format!("\nWind forces:\n"));
        s.push_str(&format!("  On boom: {:.0} lbs\n", boom_force.get::<pound_force>()));
        s.push_str(&format!("  On load: {:.0} lbs\n", load_force.get::<pound_force>()));
        s.push_str(&format!("  Overturning moment: {:.0} lb·ft\n", 
            self.wind_overturning_moment()));
        
        s
    }
}

/// Common wind speeds for reference
pub mod wind_speeds {
    use crate::types::*;
    
    /// Beaufort scale wind speeds
    
    pub fn calm() -> Velocity {
        Velocity::new::<mile_per_hour>(0.0)
    }
    
    pub fn light_breeze() -> Velocity {
        Velocity::new::<mile_per_hour>(7.0)
    }
    
    pub fn moderate_breeze() -> Velocity {
        Velocity::new::<mile_per_hour>(15.0)
    }
    
    pub fn fresh_breeze() -> Velocity {
        Velocity::new::<mile_per_hour>(22.0)
    }
    
    pub fn strong_breeze() -> Velocity {
        Velocity::new::<mile_per_hour>(30.0)
    }
    
    pub fn near_gale() -> Velocity {
        Velocity::new::<mile_per_hour>(38.0)
    }
    
    pub fn gale() -> Velocity {
        Velocity::new::<mile_per_hour>(46.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_calm_winds() {
        let analysis = WindAnalysis::new(
            CraneType::AllTerrain,
            Length::new::<foot>(150.0),
            Angle::new::<degree>(45.0),
            Area::new::<square_foot>(50.0),
            Velocity::new::<mile_per_hour>(5.0),
        );
        
        let derating = analysis.derating_factor();
        assert!(derating > 0.95);
        assert!(derating <= 1.0);
        
        assert_eq!(analysis.wind_condition(), WindCondition::Safe);
    }
    
    #[test]
    fn test_caution_winds() {
        let analysis = WindAnalysis::new(
            CraneType::AllTerrain,
            Length::new::<foot>(150.0),
            Angle::new::<degree>(45.0),
            Area::new::<square_foot>(50.0),
            Velocity::new::<mile_per_hour>(25.0),
        );
        
        let derating = analysis.derating_factor();
        assert!(derating < 0.95);
        assert!(derating > 0.0);
        
        assert_eq!(analysis.wind_condition(), WindCondition::Caution);
    }
    
    #[test]
    fn test_shutdown_winds() {
        let analysis = WindAnalysis::new(
            CraneType::AllTerrain,
            Length::new::<foot>(150.0),
            Angle::new::<degree>(45.0),
            Area::new::<square_foot>(50.0),
            Velocity::new::<mile_per_hour>(35.0),
        );
        
        let derating = analysis.derating_factor();
        assert_relative_eq!(derating, 0.0);
        
        assert_eq!(analysis.wind_condition(), WindCondition::Shutdown);
        assert!(analysis.validate_for_operation().is_err());
    }
    
    #[test]
    fn test_wind_forces() {
        let analysis = WindAnalysis::new(
            CraneType::AllTerrain,
            Length::new::<foot>(150.0),
            Angle::new::<degree>(45.0),
            Area::new::<square_foot>(50.0),
            Velocity::new::<mile_per_hour>(30.0),
        );
        
        let boom_force = analysis.wind_force_on_boom();
        let load_force = analysis.wind_force_on_load();
        
        // Should be non-zero
        assert!(boom_force.get::<pound_force>() > 0.0);
        assert!(load_force.get::<pound_force>() > 0.0);
        
        // Load force should be significant for 50 sq ft sail area
        assert!(load_force.get::<pound_force>() > 100.0);
    }
    
    #[test]
    fn test_derated_capacity() {
        let analysis = WindAnalysis::new(
            CraneType::AllTerrain,
            Length::new::<foot>(150.0),
            Angle::new::<degree>(45.0),
            Area::new::<square_foot>(50.0),
            Velocity::new::<mile_per_hour>(25.0),
        );
        
        let rated = Mass::new::<pound>(50000.0);
        let derated = analysis.derated_capacity(rated);
        
        // Should be less than rated
        assert!(derated < rated);
        
        // Should be more than zero (in caution range)
        assert!(derated.get::<pound>() > 0.0);
    }
}
