pub mod mobile;
pub mod tower;
use nalgebra as na;
use crate::types::*;
use crate::capacity::load_chart::LoadChart;
use crate::kinematics::{ForwardKinematics, JointConfig};
pub use mobile::MobileCrane;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CraneType {
    MobileTelescopic,
    MobileLattice,
    AllTerrain,
    RoughTerrain,
    Crawler,
    Tower,
    TruckMounted,
}

/// Core trait all crane types must implement
pub trait Crane {
    /// Get the crane's current configuration
    fn configuration(&self) -> CraneConfig;
    
    /// Calculate boom tip position given current joint angles
    fn tip_position(&self) -> na::Point3<Length>;
    
    /// Get the load chart for current configuration
    fn load_chart(&self) -> &LoadChart;
    
    /// Calculate center of gravity of entire crane + load system
    fn system_cog(&self, load: Mass) -> na::Point3<Length>;
    
    /// Calculate tipping moment for given load at current position
    fn tipping_moment(&self, load: Mass) -> Torque;
    
    /// Maximum rated capacity at current configuration
    fn rated_capacity(&self) -> Mass;
    
    /// Validate if lift is within safety parameters
    fn validate_lift(&self, load: Mass) -> Result<(), LiftError>;
    
    /// Get forward kinematics solver for this crane
    fn forward_kinematics(&self) -> ForwardKinematics;
    
    /// Get current joint configuration
    fn joint_config(&self) -> JointConfig;
    
    /// Set joint configuration (move crane to position)
    fn set_joint_config(&mut self, joints: JointConfig);

}

#[derive(Debug, Clone)]
pub struct CraneConfig {
    pub boom_length: Length,
    pub boom_angle: Angle,  // From horizontal
    pub radius: Length,   // Horizontal distance from centerline
    pub height: Length,   // Hook height above ground
}

#[derive(Debug, thiserror::Error)]
pub enum LiftError {
    #[error("Load {load:?} exceeds rated capacity {capacity:?}")]
    OverCapacity { load: Mass, capacity: Mass },
    
    #[error("Configuration exceeds load chart at radius {radius:?}")]
    LoadChartExceeded { radius: Length },
    
    #[error("Tipping moment {moment} exceeds stability limit {limit}")]
    TippingRisk { moment: f64, limit: f64 },
}
