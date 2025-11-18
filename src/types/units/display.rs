use std::fmt;
use uom::si::{angle::degree, force::{newton, pound_force}, length::{inch, meter}, mass::{kilogram, pound}, pressure::{bar, kilopascal, psi}, velocity::{kilometer_per_hour, mile_per_hour}};

use crate::types::units::*;
#[derive(Debug)]
pub struct DisplayForce(pub Force);
#[derive(Debug)]
pub struct DisplayMass(pub Mass);
#[derive(Debug)]
pub struct DisplayAngle(pub Angle);
#[derive(Debug)]
pub struct DisplayLength(pub Length);
#[derive(Debug)]
pub struct DisplayHydraulicPressure(pub Pressure);
#[derive(Debug)]
pub struct DisplayGroundBearingPressure(pub Pressure);
#[derive(Debug)]
pub struct DisplayVelocity(pub Velocity);

impl fmt::Display for DisplayForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lbf = self.0.get::<pound_force>();
        let newtons = self.0.get::<newton>();
        write!(f, "{:.0} lbf ({:.2}) N", lbf, newtons)
    }
}

impl fmt::Display for DisplayMass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lbs = self.0.get::<pound>();
        let kg = self.0.get::<kilogram>();
        write!(f, "{:.0} lbs ({:.0}kg)", lbs, kg)
        
    }
}

impl fmt::Display for DisplayAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}°", self.0.get::<degree>())
    }
}

impl fmt::Display for DisplayLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_inches = self.0.get::<inch>();
        let feet = (total_inches / 12.0).floor();
        let inches = total_inches - (feet * 12.0);
        let meters = self.0.get::<meter>();
        write!(f, "{}' {:.3}\" ({:.3}m)", feet, inches, meters)
    }
}

impl fmt::Display for DisplayVelocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mph_f64 = self.0.get::<mile_per_hour>();
        let kph_f64 = self.0.get::<kilometer_per_hour>();

        write!(f, "{:1}mph ({:.1})kph", mph_f64, kph_f64)
    }
}

impl fmt::Display for DisplayHydraulicPressure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let psi_f64 = self.0.get::<psi>();
        let bar_f64 = self.0.get::<bar>();
        
        write!(f, "{:.0}psi ({:.1}bar)", psi_f64, bar_f64)
    }
}

impl fmt::Display for DisplayGroundBearingPressure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let psi_f64 = self.0.get::<psi>();
        let kpa_f64 = self.0.get::<kilopascal>();

        write!(f, "{:.0}psi ({:.0}kPa)", psi_f64, kpa_f64)
    }
}

