pub use uom::si::f64::*;
pub use uom::si::{
    angle::{degree, radian}, 
    force::{newton, pound_force}, 
    length::{centimeter, foot, inch, meter, millimeter, yard}, 
    mass::{gram, kilogram, pound, ton, ton_long, ton_short}, 
    pressure::{kilopascal, pascal, psi, bar},
    area::{
        square_inch, square_foot, acre, square_mile, 
        square_millimeter, square_centimeter, square_kilometer, square_meter
    },
    velocity::{mile_per_hour, kilometer_per_hour, foot_per_minute, foot_per_second, meter_per_second, knot}
};
pub use uom::si::{length, mass, angle, force, pressure, area};
use serde::{Deserialize,  Serialize};
// Type aliases for domain clarity (zero cost)
pub type Distance = Length;
pub type Weight = Mass;
pub type LoadForce = Force;
pub type CraneAngle = Angle;
pub type HydraulicPressure = Pressure;
pub type GroundBearingPressure = Pressure;

// Common units for convenience
pub mod units {
    pub use uom::si::length::{foot, meter, inch};
    pub use uom::si::mass::{pound, kilogram, ton};
    pub use uom::si::angle::{degree, radian};
    pub use uom::si::force::{pound_force, newton};
    pub use uom::si::pressure::{pound_force_per_square_inch as psi, pascal};
}

// Re-export nalgebra
pub use nalgebra as na;

// Standard units we use internally (just documentation)
/// Internal standard: feet
pub const INTERNAL_LENGTH_UNIT: &str = "feet";
/// Internal standard: pounds
pub const INTERNAL_MASS_UNIT: &str = "pounds";
/// Internal standard: radians
pub const INTERNAL_ANGLE_UNIT: &str = "radians";

use std::{fmt, marker::PhantomData};
#[derive(Debug)]
pub struct DisplayForce(pub Force);
#[derive(Debug)]
pub struct DisplayWeight(pub Weight);
#[derive(Debug)]
pub struct DisplayAngle(pub Angle);
#[derive(Debug)]
pub struct DisplayDistance(pub Distance);
#[derive(Debug)]
pub struct DisplayHydraulicPressure(pub HydraulicPressure);
#[derive(Debug)]
pub struct DisplayGroundBearingPressure(pub GroundBearingPressure);
#[derive(Debug)]
pub struct DisplaySpeed(pub Velocity);

impl fmt::Display for DisplayForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lbf = self.0.get::<pound_force>();
        let newtons = self.0.get::<newton>();
        write!(f, "{:.0} lbf ({:.2}) N", lbf, newtons)
    }
}

impl fmt::Display for DisplayWeight {
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

impl fmt::Display for DisplayDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_inches = self.0.get::<inch>();
        let feet = (total_inches / 12.0).floor();
        let inches = total_inches - (feet * 12.0);
        let meters = self.0.get::<meter>();
        write!(f, "{}' {:.3}\" ({:.3}m)", feet, inches, meters)
    }
}

impl fmt::Display for DisplaySpeed {
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
        let psi_f64 = self.0.get::<pressure::psi>();
        let kpa_f64 = self.0.get::<pressure::kilopascal>();

        write!(f, "{:.0}psi ({:.0}kPa)", psi_f64, kpa_f64)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithUnit<T> {
    pub value: f64,
    pub unit: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

pub type LengthValue = WithUnit<Length>;
pub type WeightValue = WithUnit<Mass>;
pub type AngleValue = WithUnit<Angle>;
pub type GroundBearingPressureValue = WithUnit<Pressure>;
pub type HydraulicPressureValue = WithUnit<Pressure>;

impl<T> WithUnit<T> {
    pub fn new(value: f64, unit: impl Into<String>) -> Self {
        Self {
            value,
            unit: unit.into(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnitError {
    #[error("Unknown length unit: {0}")]
    UnknownLengthUnit(String),
    
    #[error("Unknown mass unit: {0}")]
    UnknownWeightUnit(String),
    
    #[error("Unknown angle unit: {0}")]
    UnknownAngleUnit(String),

    #[error("Unknown pressure unit: {0}")]
    UnknownPressureUnit(String)
}

impl WithUnit<Length> {
    pub fn to_distance(&self) -> Result<Distance, UnitError> {
        match self.unit.as_str() {
            "ft" | "Ft" | "FT" 
            | "foot" | "Foot" | "FOOT" 
            | "feet" | "Feet" | "FEET" => Ok(Distance::new::<foot>(self.value)),
            "in" | "In" | "IN" 
            | "inch" | "Inch" | "INCH"
            | "inches" | "Inches" | "INCHES" => Ok(Distance::new::<inch>(self.value)),
            "m" | "M" 
            | "meter" | "Meter" | "METER" 
            | "metre" | "Metre" | "METRE" 
            | "meters" | "Meters" | "METERS"
            | "metres" | "Metres" | "METRES" => Ok(Distance::new::<meter>(self.value)),
            "cm" | "Cm" | "CM"
            | "centimeter" | "Centimeter" | "CENTIMETER"
            | "centimetre" | "Centimetre" | "CENTIMETRE"
            | "centimeters" | "Centimeters" | "CENTIMETERS"
            | "centimetres" | "Centimetres" | "CENTIMETRES" => Ok(Distance::new::<centimeter>(self.value)),
            "mm" | "Mm" | "MM" 
            | "millimeter" | "Millimeter" | "MILLIMETER"
            | "millimetre" | "Millinetre" | "MILLIMETRE"
            | "millimeters" | "Millimeters" | "MILLIMETERS"
            | "millimetres" | "Millimetres" | "MILLIMETRES" => Ok(Distance::new::<millimeter>(self.value)),
            "yd" | "Yd" | "YD"
            | "yard" | "Yard" | "YARD"
            | "yards" |"Yards" | "YARDS" => Ok(Distance::new::<yard>(self.value)),
            _ => Err(UnitError::UnknownLengthUnit(self.unit.clone()))
        }
    }
    
    pub fn from_distance(distance: Distance, unit: &str) -> Result<Self, UnitError> {
        let value = match unit {
            "ft" | "Ft" | "FT" 
            | "foot" | "Foot" | "FOOT" 
            | "feet" | "Feet" | "FEET" => distance.get::<foot>(),
            "in" | "In" | "IN" 
            | "inch" | "Inch" | "INCH"
            | "inches" | "Inches" | "INCHES" => distance.get::<inch>(),
            "m" | "M" 
            | "meter" | "Meter" | "METER" 
            | "metre" | "Metre" | "METRE" 
            | "meters" | "Meters" | "METERS"
            | "metres" | "Metres" | "METRES" => distance.get::<meter>(),
            "cm" | "Cm" | "CM"
            | "centimeter" | "Centimeter" | "CENTIMETER"
            | "centimetre" | "Centimetre" | "CENTIMETRE"
            | "centimeters" | "Centimeters" | "CENTIMETERS"
            | "centimetres" | "Centimetres" | "CENTIMETRES" => distance.get::<centimeter>(),
            "mm" | "Mm" | "MM" 
            | "millimeter" | "Millimeter" | "MILLIMETER"
            | "millimetre" | "Millinetre" | "MILLIMETRE"
            | "millimeters" | "Millimeters" | "MILLIMETERS"
            | "millimetres" | "Millimetres" | "MILLIMETRES" => distance.get::<millimeter>(),
            "yd" | "Yd" | "YD"
            | "yard" | "Yard" | "YARD"
            | "yards" |"Yards" | "YARDS" => distance.get::<yard>(),
            _ => return Err(UnitError::UnknownLengthUnit(unit.to_string()))
        };
        
        Ok(Self::new(value, unit))
    }
}

impl WithUnit<Mass> {
    pub fn to_weight(&self) -> Result<Weight, UnitError> {
        match self.unit.as_str() {
            "lb" | "Lb" | "LB"
            | "lbs" | "Lbs" | "LBS"
            | "pound" | "Pound" | "POUND"
            | "pounds" | "Pounds" | "POUNDS" => Ok(Weight::new::<pound>(self.value)), 
            "kg" | "Kg" | "KG"
            | "kgs" | "Kgs" | "KGS"
            | "kilogram" | "Kilogram" | "KILOGRAN"
            | "kilograms" | "Kilograms" | "KILOGRANS" => Ok(Weight::new::<kilogram>(self.value)),
            | "short ton" | "Short Ton" | "SHORT TON"
            | "short tons" | "Short Tons" | "SHORT TONS" => Ok(Weight::new::<ton_short>(self.value)),
            "metric ton" | "Metric Ton" | "METRIC TON"
            | "metric tons" | "Metric Tons" | "METRIC TONS" => Ok(Weight::new::<ton>(self.value)),
            "long ton" | "Long Ton" | "LONG TON"
            | "long tons" | "Long Tons" | "LONG TONS" => Ok(Weight::new::<ton_long>(self.value)),
            "g" | "G"
            | "gram" | "Gram" | "GRAM"
            | "grams" | "Grams" | "GRAMS" => Ok(Weight::new::<gram>(self.value)),
            _ => Err(UnitError::UnknownWeightUnit(self.unit.clone())),
        }
    }
    
    pub fn from_weight(weight: Weight, unit: &str) -> Result<Self, UnitError> {
        let value = match unit {
            "lb" | "Lb" | "LB"
            | "lbs" | "Lbs" | "LBS"
            | "pound" | "Pound" | "POUND"
            | "pounds" | "Pounds" | "POUNDS" => weight.get::<pound>(), 
            "kg" | "Kg" | "KG"
            | "kgs" | "Kgs" | "KGS"
            | "kilogram" | "Kilogram" | "KILOGRAN"
            | "kilograms" | "Kilograms" | "KILOGRANS" => weight.get::<kilogram>(),
            | "short ton" | "Short Ton" | "SHORT TON"
            | "short tons" | "Short Tons" | "SHORT TONS" => weight.get::<ton_short>(),
            "metric ton" | "Metric Ton" | "METRIC TON"
            | "metric tons" | "Metric Tons" | "METRIC TONS" => weight.get::<ton>(),
            "long ton" | "Long Ton" | "LONG TON"
            | "long tons" | "Long Tons" | "LONG TONS" => weight.get::<ton_long>(),
            "g" | "G"
            | "gram" | "Gram" | "GRAM"
            | "grams" | "Grams" | "GRAMS" => weight.get::<gram>(),
            _ => return Err(UnitError::UnknownWeightUnit(unit.to_string()))
        };
        
        Ok(Self::new(value, unit))
    }
}

impl WithUnit<Angle> {
    pub fn to_angle(&self) -> Result<Angle, UnitError> {
        match self.unit.as_str() {
            "deg" | "Deg" | "DEG"
            | "degree" | "Degree" | "DEGREE"
            | "degrees" | "Degrees" | "DEGREES" | "°" => Ok(Angle::new::<degree>(self.value)),
            "rad" | "Rad" | "RAD"
            | "rads" | "Rads" | "RADS"
            | "radian" | "Radian" | "RADIAN"
            | "radians" | "Radians" | "RADIANS" => Ok(Angle::new::<radian>(self.value)),
            _ => Err(UnitError::UnknownAngleUnit(self.unit.clone()))
        }
    }
    
    pub fn from_angle(angle: Angle, unit: &str) -> Result<Self, UnitError> {
        let value = match unit {
            "deg" | "Deg" | "DEG"
            | "degree" | "Degree" | "DEGREE"
            | "degrees" | "Degrees" | "DEGREES" | "°" => angle.get::<degree>(),
            "rad" | "Rad" | "RAD"
            | "rads" | "Rads" | "RADS"
            | "radian" | "Radian" | "RADIAN"
            | "radians" | "Radians" | "RADIANS" => angle.get::<radian>(),
            _ => return Err(UnitError::UnknownAngleUnit(unit.to_string())),
        };
        
        Ok(Self::new(value, unit))
    }
}

impl WithUnit<GroundBearingPressure> {
    pub fn to_pressure(&self) -> Result<Pressure, UnitError> {
        match self.unit.as_str() {
            "psi" | "lbf/in^2" | "lbf/in²" 
            | "lb/in²" | "lb/in^2" | "pound per square inch"
            | "pounds per square inch" => Ok(Pressure::new::<psi>(self.value)),
            "pa" | "Pa" | "PA" 
            | "pascal" | "Pascal" | "PASCAL" 
            | "pascals" | "Pascals" | "PASCALS"
            | "N/m²" | "N/m^2" => Ok(Pressure::new::<pascal>(self.value)),
            "kilopascal" | "Kilopascal" | "KILOPASCAL"
            | "kilopascals" | "Kilopascals" | "KILOPASCALS"
            | "kPa" | "KPa" | "kPA"
            | "kPas" | "KPas" | "KPAs" | "KPAS" => Ok(Pressure::new::<kilopascal>(self.value)),
            _ => return Err(UnitError::UnknownPressureUnit(self.unit.clone())),
        }
    }

    pub fn from_pressure(pressure: Pressure, unit: &str) -> Result<Self, UnitError> {
        let value = match unit {
            "psi" | "lbf/in^2" | "lbf/in²" 
            | "lb/in²" | "lb/in^2" | "pound per square inch"
            | "pounds per square inch" => pressure.get::<psi>(),
            "pa" | "Pa" | "PA" 
            | "pascal" | "Pascal" | "PASCAL" 
            | "pascals" | "Pascals" | "PASCALS"
            | "N/m²" | "N/m^2" => pressure.get::<pascal>(),
            "kilopascal" | "Kilopascal" | "KILOPASCAL"
            | "kilopascals" | "Kilopascals" | "KILOPASCALS"
            | "kPa" | "KPa" | "kPA"
            | "kPas" | "KPas" | "KPAs" | "KPAS" => pressure.get::<kilopascal>(),
            _ => return Err(UnitError::UnknownPressureUnit(unit.to_string())),
        };

        Ok(Self::new(value, unit))
    }
}

/// Convert UOM Distance to internal coordinate (feet)
#[inline]
pub fn to_coord(distance: Distance) -> f64 {
    distance.get::<foot>()
}

/// Convert internal coordinate (feet) to UOM Distnance
#[inline]
pub fn from_coord(value: f64) -> Distance {
    Distance::new::<foot>(value)
}

/// Create Point3 from UOM Distances
pub fn point_from_distances(x: Distance, y: Distance, z: Distance) -> na::Point3<f64> {
    na::Point3::new(
        to_coord(x),
        to_coord(y),
        to_coord(z),
    )
}

/// Extract X coordinate as Distance
pub fn x_distance(point: &na::Point3<f64>) -> Distance {
    from_coord(point.x)
}

/// Extract Y coordinate as Distance
pub fn y_distance(point: &na::Point3<f64>) -> Distance {
    from_coord(point.y)
}

pub fn z_distance(point: &na::Point3<f64>) -> Distance {
    from_coord(point.z)
}
