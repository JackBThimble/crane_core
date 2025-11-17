use nalgebra as na;
use crate::types::*;
use crate::types::units::*;

/// Create a rotation matrix around Y axis (vertical, for boom angle)
pub fn rotation_y(angle: Angle) -> na::Matrix3<f64> {
    let theta = angle.get::<radian>();
    let c = theta.cos();
    let s = theta.sin();
    
    na::Matrix3::new(
        c,  0.0, s,
        0.0, 1.0, 0.0,
        -s, 0.0, c,
    )
}

/// Create a rotation matrix around Y axis (vertical, for swing/slew)
pub fn rotation_y_swing(angle: Angle) -> na::Matrix3<f64> {
    rotation_y(angle)
}

/// Create a rotation matrix around X axis (for jib offset)
pub fn rotation_x(angle: Angle) -> na::Matrix3<f64> {
    let theta = angle.get::<radian>();
    let c = theta.cos();
    let s = theta.sin();
    
    na::Matrix3::new(
        1.0, 0.0, 0.0,
        0.0, c,  -s,
        0.0, s,   c,
    )
}

/// Create a rotation matrix around Z axis
pub fn rotation_z(angle: Angle) -> na::Matrix3<f64> {
    let theta = angle.get::<radian>();
    let c = theta.cos();
    let s = theta.sin();
    
    na::Matrix3::new(
        c,  -s, 0.0,
        s,   c, 0.0,
        0.0, 0.0, 1.0,
    )
}

/// Create a 4x4 transformation matrix (rotation + translation)
pub fn transform_matrix(
    rotation: na::Matrix3<f64>,
    translation: na::Vector3<f64>,
) -> na::Matrix4<f64> {
    let mut mat = na::Matrix4::identity();
    mat.fixed_view_mut::<3, 3>(0, 0).copy_from(&rotation);
    mat.fixed_view_mut::<3, 1>(0, 3).copy_from(&translation);
    mat
}

/// Apply a transformation to a point
pub fn transform_point(
    transform: &na::Matrix4<f64>,
    point: na::Point3<f64>,
) -> na::Point3<f64> {
    let homogeneous = na::Vector4::new(point.x, point.y, point.z, 1.0);
    let transformed = transform * homogeneous;
    na::Point3::new(transformed.x, transformed.y, transformed.z)
}