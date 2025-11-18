use uom::si::{f64::Length, length::foot};
use nalgebra as na;


/// Convert UOM Distance to internal coordinate (feet)
#[inline]
pub fn to_coord(length: Length) -> f64 {
    length.get::<foot>()
}

/// Convert internal coordinate (feet) to UOM Distnance
#[inline]
pub fn from_coord(value: f64) -> Length {
    Length::new::<foot>(value)
}

/// Create Point3 from UOM Distances
pub fn point_from_uom_lengths(x: Length, y: Length, z: Length) -> na::Point3<f64> {
    na::Point3::new(
        to_coord(x),
        to_coord(y),
        to_coord(z),
    )
}

/// Extract X coordinate as Distance
pub fn x_uom_length(point: &na::Point3<f64>) -> Length {
    from_coord(point.x)
}

/// Extract Y coordinate as Distance
pub fn y_uom_length(point: &na::Point3<f64>) -> Length {
    from_coord(point.y)
}

pub fn z_uom_length(point: &na::Point3<f64>) -> Length {
    from_coord(point.z)
}
