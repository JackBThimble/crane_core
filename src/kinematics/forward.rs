use nalgebra as na;
use crate::types::*;
use crate::types::units::*;
use crate::kinematics::transforms::*;

/// Joint configuration for a crane
/// 
/// This represents the "joint space" - all the angles and extensions
#[derive(Debug, Clone, Copy)]
pub struct JointConfig {
    /// Swing/slew angle (rotation around vertical Y axis)
    pub swing: Angle,
    
    /// Boom angle from horizontal
    pub boom_angle: Angle,
    
    /// Boom length (for telescoping booms)
    pub boom_length: Distance,
    
    /// Jib configuration (if present)
    pub jib: Option<JibConfig>,
}

#[derive(Debug, Clone, Copy)]
pub struct JibConfig {
    /// Jib angle relative to boom
    pub jib_angle: Angle,
    
    /// Jib length
    pub jib_length: Distance,
    
    /// Jib offset angle (side-to-side tilt)
    pub jib_offset: Angle,
}

/// Base position of crane (where the boom pivots)
#[derive(Debug, Clone, Copy)]
pub struct CraneBase {
    /// Position of boom pivot point
    pub position: na::Point3<f64>,
    
    /// Height of boom pivot above ground
    pub pivot_height: Distance,
}

impl CraneBase {
    pub fn new(x: Distance, y: Distance, z: Distance, pivot_height: Distance) -> Self {
        Self {
            position: na::Point3::new(
                x.get::<foot>(),
                y.get::<foot>(),
                z.get::<foot>(),
            ),
            pivot_height,
        }
    }
    
    /// Get the boom pivot point in world space
    pub fn pivot_point(&self) -> na::Point3<f64> {
        na::Point3::new(
            self.position.x,
            self.position.y + self.pivot_height.get::<foot>(),
            self.position.z,
        )
    }
}

/// Forward kinematics solver
/// 
/// Given joint angles, calculate hook position
pub struct ForwardKinematics {
    /// Base position and orientation
    pub base: CraneBase,
}

impl ForwardKinematics {
    pub fn new(base: CraneBase) -> Self {
        Self { base }
    }
    
    /// Calculate hook position from joint configuration
    /// 
    /// This is the core FK calculation - transforms from joint space to task space
    pub fn solve(&self, joints: &JointConfig) -> na::Point3<f64> {
        let pivot = self.base.pivot_point();
        
        // Start at boom pivot
        let mut position = na::Vector3::zeros();
        
        // 1. Apply boom angle (rotation around X axis in local frame, but we need to 
        //    account for swing first)
        let boom_len = joints.boom_length.get::<foot>();
        let boom_angle = joints.boom_angle.get::<radian>();
        
        // Boom extends in local Z direction (forward) and Y direction (up)
        // Before swing rotation
        let boom_local = na::Vector3::new(
            0.0,
            boom_len * boom_angle.sin(),
            boom_len * boom_angle.cos(),
        );
        
        // 2. Apply swing rotation
        let swing_rot = rotation_y_swing(joints.swing);
        let boom_world = swing_rot * boom_local;
        
        position += boom_world;
        
        // 3. If there's a jib, apply jib kinematics
        if let Some(jib) = joints.jib {
            let jib_position = self.solve_jib(&jib, position, joints.swing, joints.boom_angle);
            position = jib_position;
        }
        
        // 4. Transform to world coordinates
        na::Point3::from(pivot.coords + position)
    }
    
    /// Solve jib kinematics (relative to boom tip)
    fn solve_jib(
        &self,
        jib: &JibConfig,
        boom_tip: na::Vector3<f64>,
        swing: Angle,
        boom_angle: Angle,
    ) -> na::Vector3<f64> {
        let jib_len = jib.jib_length.get::<foot>();
        
        // Jib angle is relative to boom
        // Total angle from horizontal = boom_angle + jib_angle
        let total_angle = boom_angle.get::<radian>() + jib.jib_angle.get::<radian>();
        
        // Jib position in local frame (before swing and offset)
        let jib_local = na::Vector3::new(
            0.0,
            jib_len * total_angle.sin(),
            jib_len * total_angle.cos(),
        );
        
        // Apply jib offset (rotation around boom axis)
        let offset_rot = rotation_z(jib.jib_offset);
        let jib_with_offset = offset_rot * jib_local;
        
        // Apply swing rotation
        let swing_rot = rotation_y_swing(swing);
        let jib_world = swing_rot * jib_with_offset;
        
        boom_tip + jib_world
    }
    
    /// Calculate boom tip position (without jib)
    pub fn boom_tip(&self, joints: &JointConfig) -> na::Point3<f64> {
        let mut joints_no_jib = *joints;
        joints_no_jib.jib = None;
        self.solve(&joints_no_jib)
    }
    
    /// Calculate the reach (horizontal distance from crane centerline)
    pub fn reach(&self, joints: &JointConfig) -> Distance {
        let hook = self.solve(joints);
        let base = self.base.position;
        
        let dx = hook.x - base.x;
        let dz = hook.z - base.z;
        
        Distance::new::<foot>((dx*dx + dz*dz).sqrt())
    }
    
    /// Calculate hook height above ground
    pub fn hook_height(&self, joints: &JointConfig) -> Distance {
        let hook = self.solve(joints);
        Distance::new::<foot>(hook.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_simple_boom_forward_kinematics() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0), // Pivot 10ft up
        );
        
        let fk = ForwardKinematics::new(base);
        
        // 100 ft boom at 45 degrees, no swing
        let joints = JointConfig {
            swing: Angle::new::<degree>(0.0),
            boom_angle: Angle::new::<degree>(45.0),
            boom_length: Distance::new::<foot>(100.0),
            jib: None,
        };
        
        let hook = fk.solve(&joints);
        
        // At 45 degrees:
        // Y (up) = 10 (pivot) + 100 * sin(45) ≈ 10 + 70.7 = 80.7
        // Z (forward) = 100 * cos(45) ≈ 70.7
        // X (side) = 0
        
        assert_relative_eq!(hook.x, 0.0, epsilon = 0.1);
        assert_relative_eq!(hook.y, 80.7, epsilon = 0.5);
        assert_relative_eq!(hook.z, 70.7, epsilon = 0.5);
    }
    
    #[test]
    fn test_boom_with_swing() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let fk = ForwardKinematics::new(base);
        
        // 100 ft boom at 45 degrees, 90 degree swing (should point in +X direction)
        let joints = JointConfig {
            swing: Angle::new::<degree>(90.0),
            boom_angle: Angle::new::<degree>(45.0),
            boom_length: Distance::new::<foot>(100.0),
            jib: None,
        };
        
        let hook = fk.solve(&joints);
        
        // After 90 degree swing, forward (Z) becomes side (X)
        assert_relative_eq!(hook.x, 70.7, epsilon = 0.5);
        assert_relative_eq!(hook.y, 80.7, epsilon = 0.5);
        assert_relative_eq!(hook.z, 0.0, epsilon = 0.1);
    }
    
    #[test]
    fn test_reach_calculation() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let fk = ForwardKinematics::new(base);
        
        let joints = JointConfig {
            swing: Angle::new::<degree>(0.0),
            boom_angle: Angle::new::<degree>(30.0), // Shallow angle
            boom_length: Distance::new::<foot>(100.0),
            jib: None,
        };
        
        let reach = fk.reach(&joints);
        
        // At 30 degrees: reach = 100 * cos(30) ≈ 86.6 ft
        assert_relative_eq!(reach.get::<foot>(), 86.6, epsilon = 0.5);
    }
    
    #[test]
    fn test_with_jib() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let fk = ForwardKinematics::new(base);
        
        // 80 ft boom at 60 degrees, 40 ft jib at -30 degrees relative to boom
        let joints = JointConfig {
            swing: Angle::new::<degree>(0.0),
            boom_angle: Angle::new::<degree>(60.0),
            boom_length: Distance::new::<foot>(80.0),
            jib: Some(JibConfig {
                jib_angle: Angle::new::<degree>(-30.0),
                jib_length: Distance::new::<foot>(40.0),
                jib_offset: Angle::new::<degree>(0.0),
            }),
        };
        
        let hook = fk.solve(&joints);
        let boom_tip = fk.boom_tip(&joints);
        
        // Verify jib extends from boom tip
        let jib_vec = hook - boom_tip;
        let jib_length = jib_vec.magnitude();
        
        assert_relative_eq!(jib_length, 40.0, epsilon = 0.5);
    }
}