use crate::types::*;

/// Live rigging devices that can adjust tension
#[derive(Debug, Clone)]
pub enum LiveRiggingDevice {
    /// Chain fall (chain hoist) - manual or powered
    ChainFall {
        capacity: Mass,
        lift_height: Length,
        is_powered: bool,
    },
    
    /// Lever hoist (come-along)
    LeverHoist {
        capacity: Mass,
        lift_height: Length,
        lever_ratio: f64, // Mechanical advantage
    },
    
    /// Hydraulic jack
    HydraulicJack {
        capacity: Mass,
        stroke: Length,
    },
    
    /// Winch
    Winch {
        capacity: Mass,
        line_speed: f64, // ft/min
    },
}

impl LiveRiggingDevice {
    /// Get the rated capacity of this device
    pub fn capacity(&self) -> Mass {
        match self {
            LiveRiggingDevice::ChainFall { capacity, .. } => *capacity,
            LiveRiggingDevice::LeverHoist { capacity, .. } => *capacity,
            LiveRiggingDevice::HydraulicJack { capacity, .. } => *capacity,
            LiveRiggingDevice::Winch { capacity, .. } => *capacity,
        }
    }
    
    /// Check if device can handle the given load
    pub fn can_handle(&self, load: Mass) -> bool {
        load <= self.capacity()
    }
    
    /// Calculate force required to lift (for manual devices)
    pub fn pull_force(&self, load: Mass) -> Force {
        match self {
            LiveRiggingDevice::LeverHoist { lever_ratio, .. } => {
                // Pull force = Load / mechanical advantage
                Force::new::<pound_force>(load.get::<pound>() / lever_ratio)
            }
            _ => {
                // For powered devices or other types, return full load
                Force::new::<pound_force>(load.get::<pound>())
            }
        }
    }
}

/// A leg of rigging using a live device
#[derive(Debug, Clone)]
pub struct LiveLeg {
    /// The device being used
    pub device: LiveRiggingDevice,
    
    /// Current tension being applied
    pub tension: Force,
    
    /// Attachment point on load (relative to load COG)
    pub attachment_point: nalgebra::Point3<f64>,
}

impl LiveLeg {
    pub fn new(device: LiveRiggingDevice, attachment_point: nalgebra::Point3<f64>) -> Self {
        Self {
            device,
            tension: Force::new::<pound_force>(0.0),
            attachment_point,
        }
    }
    
    /// Set the tension on this leg
    pub fn set_tension(&mut self, tension: Force) -> Result<(), LiveRiggingError> {
        let capacity_as_force = Force::new::<pound_force>(
            self.device.capacity().get::<pound>()
        );
        
        if tension > capacity_as_force {
            return Err(LiveRiggingError::OverCapacity {
                requested: DisplayForce(tension),
                capacity: DisplayForce(capacity_as_force),
            });
        }
        
        self.tension = tension;
        Ok(())
    }
    
    /// Check if this leg is within safe operating limits
    pub fn is_safe(&self) -> bool {
        let capacity = Force::new::<pound_force>(self.device.capacity().get::<pound>());
        self.tension <= capacity
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LiveRiggingError {
    #[error("Requested tension {requested} exceeds device capacity {capacity}")]
    OverCapacity {
        requested: DisplayForce,
        capacity: DisplayForce,
    },
    
    #[error("Cannot achieve load balance with given configuration")]
    UnbalancedLoad,
}

/// Common chain fall capacities per manufacturer specs
pub mod chain_fall_specs {
    use super::*;
    
    /// 1/4 ton chain fall
    pub fn quarter_ton() -> LiveRiggingDevice {
        LiveRiggingDevice::ChainFall {
            capacity: Mass::new::<pound>(500.0),
            lift_height: Length::new::<foot>(10.0),
            is_powered: false,
        }
    }
    
    /// 1/2 ton chain fall
    pub fn half_ton() -> LiveRiggingDevice {
        LiveRiggingDevice::ChainFall {
            capacity: Mass::new::<pound>(1000.0),
            lift_height: Length::new::<foot>(10.0),
            is_powered: false,
        }
    }
    
    /// 1 ton chain fall
    pub fn one_ton() -> LiveRiggingDevice {
        LiveRiggingDevice::ChainFall {
            capacity: Mass::new::<pound>(2000.0),
            lift_height: Length::new::<foot>(10.0),
            is_powered: false,
        }
    }
    
    /// 2 ton chain fall
    pub fn two_ton() -> LiveRiggingDevice {
        LiveRiggingDevice::ChainFall {
            capacity: Mass::new::<pound>(4000.0),
            lift_height: Length::new::<foot>(10.0),
            is_powered: false,
        }
    }
    
    /// 3 ton chain fall
    pub fn three_ton() -> LiveRiggingDevice {
        LiveRiggingDevice::ChainFall {
            capacity: Mass::new::<pound>(6000.0),
            lift_height: Length::new::<foot>(10.0),
            is_powered: false,
        }
    }
}
