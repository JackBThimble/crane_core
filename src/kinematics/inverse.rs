use nalgebra as na;
use crate::types::*;
use crate::types::units::*;
use crate::kinematics::forward::*;

/// Inverse kinematics solver
/// 
/// Given a desired hook position, calculate required joint angles
pub struct InverseKinematics {
    pub base: CraneBase,
    
    /// Joint limits
    pub limits: JointLimits,
}

#[derive(Debug, Clone, Copy)]
pub struct JointLimits {
    /// Minimum boom angle (typically near horizontal)
    pub boom_angle_min: Angle,
    
    /// Maximum boom angle (typically near vertical)
    pub boom_angle_max: Angle,
    
    /// Minimum boom length
    pub boom_length_min: Distance,
    
    /// Maximum boom length
    pub boom_length_max: Distance,
    
    /// Maximum swing angle (typically 360 degrees)
    pub swing_max: Angle,
}

impl Default for JointLimits {
    fn default() -> Self {
        Self {
            boom_angle_min: Angle::new::<degree>(0.0),
            boom_angle_max: Angle::new::<degree>(85.0), // Most cranes can't go vertical
            boom_length_min: Distance::new::<foot>(40.0),
            boom_length_max: Distance::new::<foot>(200.0),
            swing_max: Angle::new::<degree>(360.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IKSolution {
    pub joints: JointConfig,
    pub reachable: bool,
    pub within_limits: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum IKError {
    #[error("Target position unreachable with current boom length")]
    Unreachable,
    
    #[error("Solution violates joint limits")]
    JointLimitViolation,
    
    #[error("Multiple solutions available, ambiguous")]
    MultipleSolutions,
    
    #[error("No solution found")]
    NoSolution,
}

impl InverseKinematics {
    pub fn new(base: CraneBase, limits: JointLimits) -> Self {
        Self { base, limits }
    }
    
    /// Solve IK for a target hook position (no jib)
    /// 
    /// Returns joint configuration to reach target
    pub fn solve(&self, target: na::Point3<f64>, boom_length: Distance) -> Result<IKSolution, IKError> {
        let pivot = self.base.pivot_point();
        
        // 1. Calculate swing angle (trivial - just arctan2)
        let dx = target.x - pivot.x;
        let dz = target.z - pivot.z;
        let swing = Angle::new::<radian>(dz.atan2(dx).abs()); // Modified to ensure forward is 0
        
        // Actually, let me reconsider the swing calculation
        // If Z is forward and X is right:
        // - Target directly forward (Z+): swing = 0
        // - Target to right (X+): swing = 90
        // - Target backward (Z-): swing = 180
        // - Target to left (X-): swing = 270
        
        let swing = if dz.abs() < 1e-6 && dx.abs() < 1e-6 {
            Angle::new::<degree>(0.0) // Default to forward if at crane position
        } else {
            Angle::new::<radian>(dx.atan2(dz))
        };
        
        // 2. Calculate horizontal reach and vertical height
        let reach = (dx*dx + dz*dz).sqrt();
        let height = target.y - pivot.y;
        
        // 3. Calculate boom angle using geometry
        // We have a right triangle: horizontal leg = reach, vertical leg = height, hypotenuse = boom_length
        let boom_len = boom_length.get::<foot>();
        
        // Check if target is reachable
        let distance_to_target = (reach*reach + height*height).sqrt();
        if distance_to_target > boom_len + 1e-6 {
            return Err(IKError::Unreachable);
        }
        
        // Boom angle from horizontal
        let boom_angle = Angle::new::<radian>(height.atan2(reach));
        
        // 4. Check joint limits
        let within_limits = self.check_limits(boom_angle, boom_length, swing);
        
        let joints = JointConfig {
            swing,
            boom_angle,
            boom_length,
            jib: None,
        };
        
        Ok(IKSolution {
            joints,
            reachable: true,
            within_limits,
        })
    }
    
    /// Solve IK with telescoping boom (variable length)
    /// 
    /// Finds the boom length and angle to reach target
    pub fn solve_telescoping(&self, target: na::Point3<f64>) -> Result<IKSolution, IKError> {
        let pivot = self.base.pivot_point();
        
        // Calculate swing first
        let dx = target.x - pivot.x;
        let dz = target.z - pivot.z;
        let swing = if dz.abs() < 1e-6 && dx.abs() < 1e-6 {
            Angle::new::<degree>(0.0)
        } else {
            Angle::new::<radian>(dx.atan2(dz))
        };
        
        // Calculate required boom length
        let reach = (dx*dx + dz*dz).sqrt();
        let height = target.y - pivot.y;
        let required_length = (reach*reach + height*height).sqrt();
        
        // Check if within boom length limits
        let boom_length = Distance::new::<foot>(required_length);
        if required_length < self.limits.boom_length_min.get::<foot>() ||
           required_length > self.limits.boom_length_max.get::<foot>() {
            return Err(IKError::Unreachable);
        }
        
        // Calculate boom angle
        let boom_angle = Angle::new::<radian>(height.atan2(reach));
        
        let within_limits = self.check_limits(boom_angle, boom_length, swing);
        
        let joints = JointConfig {
            swing,
            boom_angle,
            boom_length,
            jib: None,
        };
        
        Ok(IKSolution {
            joints,
            reachable: true,
            within_limits,
        })
    }
    
    /// Solve IK with jib configuration
    /// 
    /// This is more complex - we have multiple solutions (boom+jib angle combinations)
    /// For now, we'll use a simple approach: fix the jib angle and solve for boom
    pub fn solve_with_jib(
        &self,
        target: na::Point3<f64>,
        jib_config: JibConfig,
    ) -> Result<IKSolution, IKError> {
        // This requires iterative solving or numerical optimization
        // For a 2-link IK problem, we can use the law of cosines
        
        // Simplified approach: treat boom+jib as a single effective link
        // This works if jib is fixed relative to boom
        
        // TODO: Implement proper 2-link IK solver
        // For now, return error
        Err(IKError::NoSolution)
    }
    
    /// Check if joint configuration is within limits
    fn check_limits(&self, boom_angle: Angle, boom_length: Distance, swing: Angle) -> bool {
        boom_angle >= self.limits.boom_angle_min &&
        boom_angle <= self.limits.boom_angle_max &&
        boom_length >= self.limits.boom_length_min &&
        boom_length <= self.limits.boom_length_max &&
        swing.get::<radian>().abs() <= self.limits.swing_max.get::<radian>()
    }
    
    /// Find multiple solutions for a target (if they exist)
    /// 
    /// For cranes, there's typically only one solution without jib
    /// But with telescoping, you might have boom angle vs length tradeoffs
    pub fn find_all_solutions(
        &self,
        target: na::Point3<f64>,
    ) -> Vec<IKSolution> {
        let mut solutions = Vec::new();
        
        // Try different boom lengths within limits
        let min_len = self.limits.boom_length_min.get::<foot>();
        let max_len = self.limits.boom_length_max.get::<foot>();
        let num_samples = 20;
        
        for i in 0..num_samples {
            let t = i as f64 / (num_samples - 1) as f64;
            let boom_len = Distance::new::<foot>(min_len + t * (max_len - min_len));
            
            if let Ok(solution) = self.solve(target, boom_len) {
                if solution.within_limits {
                    solutions.push(solution);
                }
            }
        }
        
        solutions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::forward::ForwardKinematics;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_simple_ik_solution() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let ik = InverseKinematics::new(base, JointLimits::default());
        
        // Target: 70 ft forward, 80 ft high (should require ~45 degree boom at 100 ft)
        let target = na::Point3::new(0.0, 80.0, 70.0);
        let boom_length = Distance::new::<foot>(100.0);
        
        let solution = ik.solve(target, boom_length).unwrap();
        
        // Verify solution is close to 45 degrees
        assert_relative_eq!(
            solution.joints.boom_angle.get::<degree>(),
            45.0,
            epsilon = 1.0
        );
        
        // Verify swing is forward (0 degrees)
        assert_relative_eq!(
            solution.joints.swing.get::<degree>(),
            0.0,
            epsilon = 1.0
        );
    }
    
    #[test]
    fn test_ik_with_swing() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let ik = InverseKinematics::new(base, JointLimits::default());
        
        // Target: 50 ft to the right (X), 80 ft high
        let target = na::Point3::new(50.0, 80.0, 0.0);
        let boom_length = Distance::new::<foot>(100.0);
        
        let solution = ik.solve(target, boom_length).unwrap();
        
        // Swing should be ~90 degrees (pointing right)
        assert_relative_eq!(
            solution.joints.swing.get::<degree>(),
            90.0,
            epsilon = 1.0
        );
    }
    
    #[test]
    fn test_ik_unreachable() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let ik = InverseKinematics::new(base, JointLimits::default());
        
        // Target way too far: 200 ft away with only 100 ft boom
        let target = na::Point3::new(0.0, 10.0, 200.0);
        let boom_length = Distance::new::<foot>(100.0);
        
        let result = ik.solve(target, boom_length);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IKError::Unreachable));
    }
    
    #[test]
    fn test_ik_telescoping() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let ik = InverseKinematics::new(base, JointLimits::default());
        
        // Target that requires specific boom length
        let target = na::Point3::new(0.0, 60.0, 80.0);
        
        let solution = ik.solve_telescoping(target).unwrap();
        
        // Verify we can reach the target by checking with FK
        let fk = ForwardKinematics::new(base);
        let achieved = fk.solve(&solution.joints);
        
        assert_relative_eq!(achieved.x, target.x, epsilon = 0.5);
        assert_relative_eq!(achieved.y, target.y, epsilon = 0.5);
        assert_relative_eq!(achieved.z, target.z, epsilon = 0.5);
    }
    
    #[test]
    fn test_roundtrip_fk_ik() {
        let base = CraneBase::new(
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(0.0),
            Distance::new::<foot>(10.0),
        );
        
        let fk = ForwardKinematics::new(base);
        let ik = InverseKinematics::new(base, JointLimits::default());
        
        // Start with joint configuration
        let original_joints = JointConfig {
            swing: Angle::new::<degree>(30.0),
            boom_angle: Angle::new::<degree>(50.0),
            boom_length: Distance::new::<foot>(120.0),
            jib: None,
        };
        
        // Forward kinematics: joints -> position
        let target = fk.solve(&original_joints);
        
        // Inverse kinematics: position -> joints
        let solution = ik.solve(target, original_joints.boom_length).unwrap();
        
        // Verify we get back the same configuration
        assert_relative_eq!(
            solution.joints.swing.get::<degree>(),
            original_joints.swing.get::<degree>(),
            epsilon = 0.5
        );
        assert_relative_eq!(
            solution.joints.boom_angle.get::<degree>(),
            original_joints.boom_angle.get::<degree>(),
            epsilon = 0.5
        );
    }
}