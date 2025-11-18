pub use uom::si::f64::*;
pub use uom::si::{length, mass, angle, force, pressure, area};

mod units;
mod conversion;
// Re-export nalgebra
pub use nalgebra as na;
pub use units::*;
pub use conversion::*;

// Standard units we use internally (just documentation)
/// Internal standard: feet
pub const INTERNAL_LENGTH_UNIT: &str = "feet";
/// Internal standard: pounds
pub const INTERNAL_MASS_UNIT: &str = "pounds";
/// Internal standard: radians
pub const INTERNAL_ANGLE_UNIT: &str = "radians";
