use serde::{self, Deserialize, Serialize};
use uom::si::{angle::{degree, radian}, f64::{Angle, Length, Mass, Pressure}, length::{centimeter, foot, inch, meter, millimeter, yard}, mass::{gram, kilogram, pound, ton, ton_long, ton_short}, pressure::{bar, kilopascal, pascal, psi}};
use std::{marker::PhantomData};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithUnit<T> {
    pub value: f64,
    pub unit: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

pub type LengthValue = WithUnit<Length>;
pub type MassValue = WithUnit<Mass>;
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
    UnknownMassUnit(String),
    
    #[error("Unknown angle unit: {0}")]
    UnknownAngleUnit(String),

    #[error("Unknown pressure unit: {0}")]
    UnknownPressureUnit(String)
}

impl WithUnit<Length> {
    pub fn to_distance(&self) -> Result<Length, UnitError> {
        match self.unit.as_str() {
            "ft" | "Ft" | "FT" 
            | "foot" | "Foot" | "FOOT" 
            | "feet" | "Feet" | "FEET" => Ok(Length::new::<foot>(self.value)),
            "in" | "In" | "IN" 
            | "inch" | "Inch" | "INCH"
            | "inches" | "Inches" | "INCHES" => Ok(Length::new::<inch>(self.value)),
            "m" | "M" 
            | "meter" | "Meter" | "METER" 
            | "metre" | "Metre" | "METRE" 
            | "meters" | "Meters" | "METERS"
            | "metres" | "Metres" | "METRES" => Ok(Length::new::<meter>(self.value)),
            "cm" | "Cm" | "CM"
            | "centimeter" | "Centimeter" | "CENTIMETER"
            | "centimetre" | "Centimetre" | "CENTIMETRE"
            | "centimeters" | "Centimeters" | "CENTIMETERS"
            | "centimetres" | "Centimetres" | "CENTIMETRES" => Ok(Length::new::<centimeter>(self.value)),
            "mm" | "Mm" | "MM" 
            | "millimeter" | "Millimeter" | "MILLIMETER"
            | "millimetre" | "Millinetre" | "MILLIMETRE"
            | "millimeters" | "Millimeters" | "MILLIMETERS"
            | "millimetres" | "Millimetres" | "MILLIMETRES" => Ok(Length::new::<millimeter>(self.value)),
            "yd" | "Yd" | "YD"
            | "yard" | "Yard" | "YARD"
            | "yards" |"Yards" | "YARDS" => Ok(Length::new::<yard>(self.value)),
            _ => Err(UnitError::UnknownLengthUnit(self.unit.clone()))
        }
    }
    
    pub fn from_length(length: Length, unit: &str) -> Result<Self, UnitError> {
        let value = match unit {
            "ft" | "Ft" | "FT" 
            | "foot" | "Foot" | "FOOT" 
            | "feet" | "Feet" | "FEET" => length.get::<foot>(),
            "in" | "In" | "IN" 
            | "inch" | "Inch" | "INCH"
            | "inches" | "Inches" | "INCHES" => length.get::<inch>(),
            "m" | "M" 
            | "meter" | "Meter" | "METER" 
            | "metre" | "Metre" | "METRE" 
            | "meters" | "Meters" | "METERS"
            | "metres" | "Metres" | "METRES" => length.get::<meter>(),
            "cm" | "Cm" | "CM"
            | "centimeter" | "Centimeter" | "CENTIMETER"
            | "centimetre" | "Centimetre" | "CENTIMETRE"
            | "centimeters" | "Centimeters" | "CENTIMETERS"
            | "centimetres" | "Centimetres" | "CENTIMETRES" => length.get::<centimeter>(),
            "mm" | "Mm" | "MM" 
            | "millimeter" | "Millimeter" | "MILLIMETER"
            | "millimetre" | "Millinetre" | "MILLIMETRE"
            | "millimeters" | "Millimeters" | "MILLIMETERS"
            | "millimetres" | "Millimetres" | "MILLIMETRES" => length.get::<millimeter>(),
            "yd" | "Yd" | "YD"
            | "yard" | "Yard" | "YARD"
            | "yards" |"Yards" | "YARDS" => length.get::<yard>(),
            _ => return Err(UnitError::UnknownLengthUnit(unit.to_string()))
        };
        
        Ok(Self::new(value, unit))
    }
}

impl WithUnit<Mass> {
    pub fn to_mass(&self) -> Result<Mass, UnitError> {
        match self.unit.as_str() {
            "lb" | "Lb" | "LB"
            | "lbs" | "Lbs" | "LBS"
            | "pound" | "Pound" | "POUND"
            | "pounds" | "Pounds" | "POUNDS" => Ok(Mass::new::<pound>(self.value)), 
            "kg" | "Kg" | "KG"
            | "kgs" | "Kgs" | "KGS"
            | "kilogram" | "Kilogram" | "KILOGRAN"
            | "kilograms" | "Kilograms" | "KILOGRANS" => Ok(Mass::new::<kilogram>(self.value)),
            | "short ton" | "Short Ton" | "SHORT TON"
            | "short tons" | "Short Tons" | "SHORT TONS" => Ok(Mass::new::<ton_short>(self.value)),
            "metric ton" | "Metric Ton" | "METRIC TON"
            | "metric tons" | "Metric Tons" | "METRIC TONS" => Ok(Mass::new::<ton>(self.value)),
            "long ton" | "Long Ton" | "LONG TON"
            | "long tons" | "Long Tons" | "LONG TONS" => Ok(Mass::new::<ton_long>(self.value)),
            "g" | "G"
            | "gram" | "Gram" | "GRAM"
            | "grams" | "Grams" | "GRAMS" => Ok(Mass::new::<gram>(self.value)),
            _ => Err(UnitError::UnknownMassUnit(self.unit.clone())),
        }
    }
    
    pub fn from_mass(mass: Mass, unit: &str) -> Result<Self, UnitError> {
        let value = match unit {
            "lb" | "Lb" | "LB"
            | "lbs" | "Lbs" | "LBS"
            | "pound" | "Pound" | "POUND"
            | "pounds" | "Pounds" | "POUNDS" => mass.get::<pound>(), 
            "kg" | "Kg" | "KG"
            | "kgs" | "Kgs" | "KGS"
            | "kilogram" | "Kilogram" | "KILOGRAN"
            | "kilograms" | "Kilograms" | "KILOGRANS" => mass.get::<kilogram>(),
            | "short ton" | "Short Ton" | "SHORT TON"
            | "short tons" | "Short Tons" | "SHORT TONS" => mass.get::<ton_short>(),
            "metric ton" | "Metric Ton" | "METRIC TON"
            | "metric tons" | "Metric Tons" | "METRIC TONS" => mass.get::<ton>(),
            "long ton" | "Long Ton" | "LONG TON"
            | "long tons" | "Long Tons" | "LONG TONS" => mass.get::<ton_long>(),
            "g" | "G"
            | "gram" | "Gram" | "GRAM"
            | "grams" | "Grams" | "GRAMS" => mass.get::<gram>(),
            _ => return Err(UnitError::UnknownMassUnit(unit.to_string()))
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

impl WithUnit<Pressure> {
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
            "bar" | "BAR" => Ok(Pressure::new::<bar>(self.value)),
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
            "bar" | "BAR" => pressure.get::<bar>(),
            _ => return Err(UnitError::UnknownPressureUnit(unit.to_string())),
        };

        Ok(Self::new(value, unit))
    }
}


