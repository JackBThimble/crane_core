use crate::types::*;

/// Dynamic load calculations (for future implementation)
/// 
/// Most crane lifts are treated as static, but dynamics matter for:
/// - Load swing during crane rotation
/// - Sudden stops (shock loading)
/// - Wind loading
/// - Acceleration/deceleration effects

/// Calculate dynamic amplification factor (DAF) for sudden loading
/// 
/// DAF accounts for impact/shock loading
/// Typical values: 1.15 for smooth lifts, 1.33 for shock loading
pub fn dynamic_amplification_factor(lift_type: LiftType) -> f64 {
    match lift_type {
        LiftType::Smooth => 1.15,
        LiftType::Normal => 1.25,
        LiftType::Shock => 1.33,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LiftType {
    /// Smooth, controlled lift
    Smooth,
    /// Normal operational lift
    Normal,
    /// Sudden loading or shock
    Shock,
}

/// Calculate pendulum period for a suspended load
/// 
/// T = 2π√(L/g)
/// where L is cable length, g is gravity
pub fn pendulum_period(cable_length: Length) -> f64 {
    let l = cable_length.get::<foot>();
    let g = 32.174; // ft/s² (gravity)
    
    2.0 * std::f64::consts::PI * (l / g).sqrt()
}

// TODO: Implement full swing dynamics when needed
// TODO: Wind loading calculations
// TODO: Acceleration-based load shifts
