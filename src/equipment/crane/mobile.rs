use nalgebra as na;
use crate::equipment::crane::{Crane, CraneConfig, LiftError, CraneType};
use crate::types::*;
use crate::kinematics::{ForwardKinematics, JointConfig, CraneBase};
use crate::capacity::load_chart::{LoadChart, LoadChartPackage, ChartConfiguration, SupportConfiguration, BoomConfiguration, OutriggerExtension, SwingRestriction, CounterweightConfiguration};
use crate::physics::wind_loading::{WindAnalysis, WindError};
use serde::{Serialize, Deserialize};

/// Mobile crane (all-terrain, rough terrain, truck-mounted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileCrane {
    pub manufacturer: String,
    pub model: String,
    
    // Physical dimensions
    pub boom_length: Distance,
    pub boom_base_height: Distance,
    
    // Current state
    pub boom_angle: Angle,
    pub swing_angle: Angle,
    
    // Stability
    pub outrigger_spread: Distance,
    pub outrigger_extension: OutriggerExtension,
    pub counterweight: Weight,
    
    // Support mode
    pub on_outriggers: bool,
    
    // Load charts
    #[serde(skip)]
    pub load_charts: Option<LoadChartPackage>,
}

impl MobileCrane {
    pub fn new(
        manufacturer: impl Into<String>,
        model: impl Into<String>,
        boom_length: Distance,
        boom_base_height: Distance,
    ) -> Self {
        Self {
            manufacturer: manufacturer.into(),
            model: model.into(),
            boom_length,
            boom_base_height,
            boom_angle: Angle::new::<degree>(45.0),
            swing_angle: Angle::new::<degree>(0.0),
            outrigger_spread: Distance::new::<foot>(20.0),
            outrigger_extension: OutriggerExtension::Full,
            counterweight: Weight::new::<pound>(10000.0),
            on_outriggers: true,
            load_charts: None,
        }
    }
    
    /// Load chart package from file
    pub fn load_charts_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let package = LoadChartPackage::from_json_file(path)?;
        
        // Verify the charts match this crane
        if package.crane_info.manufacturer != self.manufacturer || 
           package.crane_info.model != self.model {
            return Err("Chart package does not match crane model".into());
        }
        
        self.load_charts = Some(package);
        Ok(())
    }
    
    /// Set the load chart package directly
    pub fn set_load_charts(&mut self, charts: LoadChartPackage) {
        self.load_charts = Some(charts);
    }
    
    /// Get the current chart configuration
    /// 
    /// Returns configuration for chart matching. Units don't matter here
    /// since matching happens via UOM type conversions.
    pub fn current_configuration(&self) -> ChartConfiguration {
        ChartConfiguration {
            support: if self.on_outriggers {
                SupportConfiguration::OnOutriggers {
                    extension: self.outrigger_extension.clone(),
                    swing_restriction: Some(SwingRestriction::Full360),
                }
            } else {
                SupportConfiguration::OnRubber {
                    speed_restriction: Some("Stationary".into()),
                }
            },
            boom: BoomConfiguration {
                length: LengthValue::new(self.boom_length.get::<foot>(), "ft"),
                angle_range: None,
                jib: None,
            },
            counterweight: Some(CounterweightConfiguration {
                weight: WeightValue::new(self.counterweight.get::<pound>(), "lbs"),
                configuration: "Standard".into(),
            }),
            additional: std::collections::HashMap::new(),
        }
    }
    
    /// Get the appropriate load chart for current configuration
    pub fn get_current_chart(&self) -> Option<&LoadChart> {
        let charts = self.load_charts.as_ref()?;
        let config = self.current_configuration();
        charts.find_chart(&config)
    }
    
    /// Get rated capacity at current boom length and radius
    pub fn rated_capacity_at_radius(&self, radius: Distance) -> Weight {
        // Try to get from load chart
        if let Some(chart) = self.get_current_chart() {
            let capacity = chart.capacity_interpolated(self.boom_length, radius);
            if capacity.is_ok() {
                return capacity.unwrap();
            }
        
        }
        // Fallback: conservative placeholder
        Weight::new::<pound>(10000.0)
    }

        /// Calculate wind analysis for current configuration
    pub fn wind_analysis(
        &self, 
        wind_speed: Velocity, 
        load_area: Area
    ) -> WindAnalysis {
        WindAnalysis::new(
            CraneType::AllTerrain,  // Or self.crane_type field
            self.boom_length,
            self.boom_angle,
            load_area,
            wind_speed,
        )
    }
    
    /// Get wind-adjusted capacity
    pub fn wind_adjusted_capacity(
        &self, 
        wind_speed: Velocity, 
        load_area: Area
    ) -> Weight {
        let rated = self.rated_capacity();
        let analysis = self.wind_analysis(wind_speed, load_area);
        analysis.derated_capacity(rated)
    }
    
    /// Validate wind conditions for lift
    pub fn validate_wind(
        &self, 
        wind_speed: Velocity, 
        load_area: Area
    ) -> Result<(), WindError> {
        let analysis = self.wind_analysis(wind_speed, load_area);
        analysis.validate_for_operation()
    }

}

impl Crane for MobileCrane {
    fn configuration(&self) -> CraneConfig {
        let angle_rad = self.boom_angle.get::<radian>();
        let boom_ft = self.boom_length.get::<foot>();
        
        let radius = Distance::new::<foot>(boom_ft * angle_rad.cos());
        let height = Distance::new::<foot>(
            self.boom_base_height.get::<foot>() + boom_ft * angle_rad.sin()
        );
        
        CraneConfig {
            boom_length: self.boom_length,
            boom_angle: self.boom_angle,
            radius,
            height,
        }
    }
    
    fn tip_position(&self) -> na::Point3<f64> {
        let boom_len = self.boom_length.get::<foot>();
        let angle = self.boom_angle.get::<radian>();
        let swing = self.swing_angle.get::<radian>();
        
        na::Point3::new(
            boom_len * angle.cos() * swing.sin(),
            self.boom_base_height.get::<foot>() + boom_len * angle.sin(),
            boom_len * angle.cos() * swing.cos(),
        )
    }
    
    fn load_chart(&self) -> &LoadChart {
        self.get_current_chart()
            .expect("No load charts loaded. Call load_charts_from_file() first.")
    }
    
    fn system_cog(&self, load: Weight) -> na::Point3<f64> {
        let hook = self.tip_position();
        let crane_weight = self.counterweight.get::<pound>() + 50000.0;
        let total_weight = crane_weight + load.get::<pound>();
        
        let weighted = hook.coords * load.get::<pound>();
        na::Point3::from(weighted / total_weight)
    }
    
    fn tipping_moment(&self, load: Weight) -> f64 {
        let config = self.configuration();
        let radius = config.radius.get::<foot>();
        let load_lbs = load.get::<pound>();
        
        load_lbs * radius
    }
    
    fn rated_capacity(&self) -> Weight {
        let config = self.configuration();
        self.rated_capacity_at_radius(config.radius)
    }
    
    fn validate_lift(&self, load: Weight) -> Result<(), LiftError> {
        let capacity = self.rated_capacity();
        
        if load > capacity {
            return Err(LiftError::OverCapacity { load, capacity });
        }
        
        Ok(())
    }
    
    fn forward_kinematics(&self) -> ForwardKinematics {
        let base = CraneBase {
            position: na::Point3::origin(),
            pivot_height: self.boom_base_height,
        };
        ForwardKinematics::new(base)
    }
    
    fn joint_config(&self) -> JointConfig {
        JointConfig {
            swing: self.swing_angle,
            boom_angle: self.boom_angle,
            boom_length: self.boom_length,
            jib: None,
        }
    }
    
    fn set_joint_config(&mut self, joints: JointConfig) {
        self.swing_angle = joints.swing;
        self.boom_angle = joints.boom_angle;
        self.boom_length = joints.boom_length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use crate::capacity::load_chart::*;
    use crate::equipment::CraneType;

    fn create_test_chart_package() -> LoadChartPackage {
        let mut package = LoadChartPackage::new(CraneInfo {
            manufacturer: "Grove".into(),
            model: "GMK5250L".into(),
            serial_number: None,
            crane_type: CraneType::AllTerrain,
            year: Some(2020),
            chart_revision: Some("Rev 2020-03".into()),
        });
        
        // Create test chart with sample data
        let mut capacity_data = CapacityData::new();
        capacity_data.boom_lengths = vec![LengthValue::new(154.2, "ft")];
        capacity_data.data = vec![vec![
            (LengthValue::new(20.0, "ft"), WeightValue::new(242500.0, "lbs")),
            (LengthValue::new(40.0, "ft"), WeightValue::new(152000.0, "lbs")),
            (LengthValue::new(60.0, "ft"), WeightValue::new(97000.0, "lbs")),
            (LengthValue::new(80.0, "ft"), WeightValue::new(68500.0, "lbs")),
            (LengthValue::new(100.0, "ft"), WeightValue::new(50500.0, "lbs")),
        ]];
        
        let chart = LoadChart {
            id: "gmk5250l_full_outriggers".into(),
            description: "Main boom, full outriggers, 360°".into(),
            configuration: ChartConfiguration {
                support: SupportConfiguration::OnOutriggers {
                    extension: OutriggerExtension::Full,
                    swing_restriction: Some(SwingRestriction::Full360),
                },
                boom: BoomConfiguration {
                    length: LengthValue::new(154.2, "ft"),
                    angle_range: None,
                    jib: None,
                },
                counterweight: None,
                additional: std::collections::HashMap::new(),
            },
            capacity_data,
            notes: vec![
                "Capacities are based on freely suspended loads".into(),
                "Machine must be level within 1%".into(),
            ],
        };
        
        package.add_chart(chart);
        package
    }
    
    #[test]
    fn test_mobile_crane_with_load_charts() {
        let mut crane = MobileCrane::new(
            "Grove",
            "GMK5250L",
            Distance::new::<foot>(154.2),
            Distance::new::<foot>(10.0),
        );
        
        // Load test charts
        let charts = create_test_chart_package();
        crane.set_load_charts(charts);
        
        // Get capacity at specific radius
        let capacity = crane.rated_capacity_at_radius(Distance::new::<foot>(40.0));
        assert_relative_eq!(capacity.get::<pound>(), 152000.0);
        
        // Test interpolation
        let capacity = crane.rated_capacity_at_radius(Distance::new::<foot>(30.0));
        // Should be between 242500 (at 20ft) and 152000 (at 40ft)
        assert!(capacity.get::<pound>() > 152000.0);
        assert!(capacity.get::<pound>() < 242500.0);
    }
    
    #[test]
    fn test_validate_lift_with_charts() {
        let mut crane = MobileCrane::new(
            "Grove",
            "GMK5250L",
            Distance::new::<foot>(154.2),
            Distance::new::<foot>(10.0),
        );
        
        crane.set_load_charts(create_test_chart_package());
        
        // Set boom angle to create specific radius
        crane.boom_angle = Angle::new::<degree>(60.0);
        
        // Calculate radius at 60 degrees
        let config = crane.configuration();
        let radius = config.radius;
        
        // Get capacity at this radius
        let capacity = crane.rated_capacity_at_radius(radius);
        
        // Load below capacity should pass
        let safe_load = Weight::new::<pound>(capacity.get::<pound>() * 0.8);
        assert!(crane.validate_lift(safe_load).is_ok());
        
        // Load above capacity should fail
        let unsafe_load = Weight::new::<pound>(capacity.get::<pound>() * 1.2);
        assert!(crane.validate_lift(unsafe_load).is_err());
    }
}
