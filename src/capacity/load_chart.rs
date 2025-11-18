use crate::types::*;
use crate::equipment::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete load chart package for a crane model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadChartPackage {
    /// Crane identification
    pub crane_info: CraneInfo,

    /// All available load charts for this crane
    pub charts: Vec<LoadChart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraneInfo {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub crane_type: CraneType,
    pub year: Option<u32>,
    pub chart_revision: Option<String>,
}


/// A single load chart for a specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadChart {
    /// Unique identifier for this chart
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Configuration this chart applies to
    pub configuration: ChartConfiguration,

    /// The actual capacity data
    pub capacity_data: CapacityData,

    /// Notes and warnings
    pub notes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadChartError {
    #[error("Unit conversion error: {0}")]
    UnitError(#[from] UnitError),

    #[error("Boom length {0} ft not found")]
    BoomLengthNotFound(DisplayLength),

    #[error("Radius {0} ft out of range")]
    RadiusOutOfRange(DisplayLength),

    #[error("No data available for interpolation")]
    NoData,
}

/// Configuration parameters that determine which chart to use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfiguration {
    /// Support configuration
    pub support: SupportConfiguration,

    /// Boom configuration
    pub boom: BoomConfiguration,

    /// Counterweight configuration
    pub counterweight: Option<CounterweightConfiguration>,

    /// Additional configurations
    pub additional: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportConfiguration {
    /// On tires/rubber
    OnRubber {
        /// Travel speed restrictions (if any)
        speed_restriction: Option<String>,
    },

    /// On outriggers
    OnOutriggers {
        /// Outrigger extension
        extension: OutriggerExtension,

        /// 360° or over-side/over-rear restrictions
        swing_restriction: Option<SwingRestriction>,
    },

    /// On crawler tracks
    OnCrawlers {
        /// Track configuration
        track_config: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutriggerExtension {
    Full,
    Intermediate { percent: f64 },
    Minimum,
    Custom { distance: LengthValue },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwingRestriction {
    Full360,
    OverFront,
    OverRear,
    OverSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoomConfiguration {
    /// Boom length (raw value)
    pub length: LengthValue,

    /// Boom angle range (if specified)
    pub angle_range: Option<AngleRange>,

    /// Jib configuration (if present)
    pub jib: Option<JibConfiguration>,
}

/// TODO: DO WE NEED THIS???
impl BoomConfiguration {
    pub fn length_distance(&self) -> Result<Length, UnitError> {
        self.length.to_distance()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngleRange {
    pub min: AngleValue,
    pub max: AngleValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JibConfiguration {
    pub length: LengthValue,
    pub angle: AngleValue,
    pub offset: Option<AngleValue>,
}

impl JibConfiguration {
    pub fn length_distance(&self) -> Result<Length, UnitError> {
        self.length.to_distance()
    }

    pub fn angle_value(&self) -> Result<Angle, UnitError> {
        self.angle.to_angle()
    }

    //pub fn offset_value(&self) -> Result<Angle, UnitError> {
    //    self.offset.unwrap_or(None).to_angle()
    //}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterweightConfiguration {
    /// Mass  (weight)
    pub weight: MassValue,
    pub configuration: String,
}

impl CounterweightConfiguration {
    /// Get weight as UOM Mass type
    pub fn to_uom_mass(&self) -> Result<Mass, UnitError> {
        self.weight.to_mass()
    }
}

/// Capacity data stored as raw f64 values
/// Units are specified in the parent LoadChart's `units` field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityData {
    /// Boom lengths (in units specified by chart.units.length)
    pub boom_lengths: Vec<LengthValue>,

    /// For each boom length, a list of (radius, capacity) pairs
    /// Units specified by chart.units
    pub data: Vec<Vec<(LengthValue, MassValue)>>,
}

impl CapacityData {
    /// Create new empty capacity data
    pub fn new() -> Self {
        Self {
            boom_lengths: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Get capacity at exact radius for a given boom length index
    /// Returns raw value in chart units
    pub fn capacity_at(
        &self,
        boom_idx: usize,
        radius: Length,
        epsilon: Option<f64>,
    ) -> Result<Option<Mass>, UnitError> {
        let eps = epsilon.unwrap_or(0.1);
        for (r_val, w_val) in &self.data[boom_idx] {
            let r = r_val.to_distance()?;
            if (r.get::<foot>() - radius.get::<foot>()).abs() < eps {
                return Ok(Some(w_val.to_mass()?));
            }
        }
        Ok(None)
    }

    /// Get all boom lengths as UOM types
    pub fn boom_lengths(&self) -> Result<Vec<Length>, UnitError> {
        self.boom_lengths.iter().map(|v| v.to_distance()).collect()
    }

    pub fn capacity_points(&self, boom_idx: usize) -> Result<Vec<(Length, Mass)>, UnitError> {
        self.data[boom_idx]
            .iter()
            .map(|(r, w)| Ok((r.to_distance()?, w.to_mass()?)))
            .collect()
    }
    /// Get all radii for a given boom length (raw values)
    pub fn radii_for_boom(&self, boom_idx: usize) -> Result<Vec<Length>, UnitError> {
        self.data[boom_idx]
            .iter()
            .map(|(r, _)| Ok(r.to_distance()?))
            .collect()
    }
}

impl LoadChart {
    /// Get capacity at exact boom length and radius (converts to/from chart units)
    pub fn capacity_exact(
        &self,
        boom_length: Length,
        radius: Length,
    ) -> Result<Mass, LoadChartError> {
        let booms = self.capacity_data.boom_lengths()?;
        let boom_idx = booms
            .iter()
            .position(|&b| (b - boom_length).abs().get::<foot>() < 0.01)
            .ok_or_else(|| LoadChartError::BoomLengthNotFound(DisplayLength(boom_length)))?;

        let points = self.capacity_data.capacity_points(boom_idx)?;
        for (r, w) in points {
            if (r - radius).abs().get::<foot>() < 0.01 {
                return Ok(w);
            }
        }
        Err(LoadChartError::RadiusOutOfRange(DisplayLength(radius)))
    }

    /// Get interpolated capacity at any boom length and radius
    pub fn capacity_interpolated(
        &self,
        boom_length: Length,
        radius: Length,
    ) -> Result<Mass, LoadChartError> {
        // Find surrounding boom lengths
        let (boom_lower_idx, boom_upper_idx) = self.find_boom_bounds(boom_length)?;

        // Interpolate capacity at each boom length
        let capacity_lower = self.interpolate_radius(boom_lower_idx, radius)?;
        let capacity_upper = self.interpolate_radius(boom_upper_idx, radius)?;

        // If boom lengths are the same, no need to interpolate
        if boom_lower_idx == boom_upper_idx {
            return Ok(capacity_lower);
        }

        // Bilinear interpolation between boom lengths
        let booms = self.capacity_data.boom_lengths()?;
        let boom_lower_val = booms[boom_lower_idx];
        let boom_upper_val = booms[boom_upper_idx];
        let ratio = (boom_length - boom_lower_val) / (boom_upper_val - boom_lower_val);

        Ok(capacity_lower + ratio * (capacity_upper - capacity_lower))
    }

    /// Find the indices of boom lengths that bound the requested boom length
    fn find_boom_bounds(&self, boom_length: Length) -> Result<(usize, usize), LoadChartError> {
        let booms = self.capacity_data.boom_lengths()?;

        if booms.is_empty() {
            return Err(LoadChartError::NoData);
        }

        let epsilon = Length::new::<foot>(0.1);

        let lower_idx = booms
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b <= boom_length + epsilon)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .ok_or_else(|| LoadChartError::BoomLengthNotFound(DisplayLength(boom_length)))?;

        let upper_idx = booms
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b >= boom_length - epsilon)
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .ok_or_else(|| LoadChartError::BoomLengthNotFound(DisplayLength(boom_length)))?;

        Ok((lower_idx, upper_idx))
    }

    /// Interpolate capacity at a given radius for a specific boom length
    fn interpolate_radius(
        &self,
        boom_idx: usize,
        radius: Length,
    ) -> Result<Mass, LoadChartError> {
        let points = &self.capacity_data.capacity_points(boom_idx)?;

        if points.is_empty() {
            return Err(LoadChartError::NoData);
        }

        let epsilon = Length::new::<foot>(0.1);

        // Find lower radius
        let lower = points
            .iter()
            .filter(|(r, _)| *r <= radius + epsilon)
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .ok_or_else(|| LoadChartError::RadiusOutOfRange(DisplayLength(radius)))?;

        // Find upper radius
        let upper = points
            .iter()
            .filter(|(r, _)| *r >= radius - epsilon)
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .ok_or_else(|| LoadChartError::RadiusOutOfRange(DisplayLength(radius)))?;

        // If radii are the same, no interpolation needed
        if (lower.0 - upper.0).abs() < epsilon {
            return Ok(lower.1);
        }

        // Linear interpolation
        let ratio = (radius - lower.0) / (upper.0 - lower.0);
        let cap = lower.1 + ratio * (upper.1 - lower.1);

        Ok(cap)
    }

    /// Check if this chart matches the given configuration
    pub fn matches_configuration(&self, config: &ChartConfiguration) -> bool {
        // Compare support configuration
        if !self.configuration.support.matches(&config.support) {
            return false;
        }

        // Compare boom configuration
        if !self.configuration.boom.matches(&config.boom) {
            return false;
        }

        // Compare counterweight if specified
        if let (Some(my_cw), Some(req_cw)) =
            (&self.configuration.counterweight, &config.counterweight)
        {
            if !my_cw.matches(req_cw) {
                return false;
            }
        }

        true
    }

    /// Get all boom lengths as UOM types
    pub fn boom_lengths(&self) -> Result<Vec<Length>, LoadChartError> {
        let boom_lens = self.capacity_data.boom_lengths()?;
        if boom_lens.is_empty() {
            return Err(LoadChartError::NoData);
        }

        Ok(boom_lens)
    }

    /// Get capacity data for a specific boom length
    pub fn capacity_points(
        &self,
        boom_idx: usize,
    ) -> Result<Vec<(Length, Mass)>, LoadChartError> {
        let points = self.capacity_data.capacity_points(boom_idx)?;

        if points.is_empty() {
            return Err(LoadChartError::NoData);
        }

        Ok(points)
    }

    /// Check if boom length is within chart bounds
    pub fn is_boom_valid(&self, boom_length: Length) -> Result<bool, LoadChartError> {
        let booms = self.capacity_data.boom_lengths()?;
        if booms.is_empty() {
            return Ok(false);
        }

        let min = booms
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
            .unwrap();
        let max = booms
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater))
            .unwrap();

        Ok(boom_length >= *min && boom_length <= *max)
    }

    /// Check if radius is valid for given boom length
    pub fn is_radius_valid(
        &self,
        boom_length: Length,
        radius: Length,
    ) -> Result<bool, LoadChartError> {
        let (lower_idx, upper_idx) = self.find_boom_bounds(boom_length)?;

        // Check both surrounding boom lengths
        for idx in [lower_idx, upper_idx] {
            let points = self.capacity_data.capacity_points(idx)?;
            if points.is_empty() {
                continue;
            }

            let radii: Vec<Length> = points.iter().map(|(r, _)| *r).collect();
            let min = radii
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
                .unwrap();
            let max = radii
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater))
                .unwrap();

            if radius >= *min && radius <= *max {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get valid radius range for a given boom length
    pub fn radius_range(
        &self,
        boom_length: Length,
    ) -> Result<(Length, Length), LoadChartError> {
        let (lower_idx, upper_idx) = self.find_boom_bounds(boom_length)?;

        let mut all_radii = Vec::new();
        for idx in [lower_idx, upper_idx] {
            let points = self.capacity_data.capacity_points(idx)?;
            all_radii.extend(points.iter().map(|(r, _)| *r));
        }

        let min = all_radii
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
            .ok_or(LoadChartError::NoData)?;
        let max = all_radii
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater))
            .ok_or(LoadChartError::NoData)?;

        Ok((*min, *max))
    }

    /// Get maximum capacity in entire chart
    pub fn max_capacity(&self) -> Result<Mass, LoadChartError> {
        let mut max_cap = Mass::new::<pound>(0.0);

        for boom_idx in 0..self.capacity_data.boom_lengths.len() {
            let points = self.capacity_data.capacity_points(boom_idx)?;
            for (_, weight) in points {
                if weight > max_cap {
                    max_cap = weight;
                }
            }
        }

        if max_cap.get::<pound>() == 0.0 {
            return Err(LoadChartError::NoData);
        }

        Ok(max_cap)
    }

    /// Get minimum radius (closest to crane) across all boom lengths
    pub fn min_radius(&self) -> Result<Length, LoadChartError> {
        let mut min_rad = Length::new::<foot>(f64::MAX);

        for boom_idx in 0..self.capacity_data.boom_lengths.len() {
            let points = self.capacity_data.capacity_points(boom_idx)?;
            for (radius, _) in points {
                if radius < min_rad {
                    min_rad = radius;
                }
            }
        }

        if min_rad.get::<foot>() == f64::MAX {
            return Err(LoadChartError::NoData);
        }

        Ok(min_rad)
    }

    /// Get maximum radius (furthest from crane) across all boom lengths
    pub fn max_radius(&self) -> Result<Length, LoadChartError> {
        let mut max_rad = Length::new::<foot>(0.0);

        for boom_idx in 0..self.capacity_data.boom_lengths.len() {
            let points = self.capacity_data.capacity_points(boom_idx)?;
            for (radius, _) in points {
                if radius > max_rad {
                    max_rad = radius;
                }
            }
        }

        if max_rad.get::<foot>() == 0.0 {
            return Err(LoadChartError::NoData);
        }

        Ok(max_rad)
    }

    /// Get boom length range
    pub fn boom_range(&self) -> Result<(Length, Length), LoadChartError> {
        let booms = self.capacity_data.boom_lengths()?;

        let min = booms
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
            .ok_or(LoadChartError::NoData)?;
        let max = booms
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater))
            .ok_or(LoadChartError::NoData)?;

        Ok((*min, *max))
    }

    /// Validate a lift configuration is within chart bounds
    pub fn validate_bounds(
        &self,
        boom_length: Length,
        radius: Length,
    ) -> Result<(), LoadChartError> {
        if !self.is_boom_valid(boom_length)? {
            return Err(LoadChartError::BoomLengthNotFound(DisplayLength(
                boom_length,
            )));
        }

        if !self.is_radius_valid(boom_length, radius)? {
            return Err(LoadChartError::RadiusOutOfRange(DisplayLength(radius)));
        }

        Ok(())
    }

    /// Apply a derating factor (for wind, side loading, etc.)
    pub fn derated_capacity(
        &self,
        boom_length: Length,
        radius: Length,
        factor: f64,
    ) -> Result<Mass, LoadChartError> {
        let capacity = self.capacity_interpolated(boom_length, radius)?;
        Ok(Mass::new::<pound>(capacity.get::<pound>() * factor))
    }
}

// Helper trait for matching configurations
trait ConfigurationMatch {
    fn matches(&self, other: &Self) -> bool;
}

impl ConfigurationMatch for SupportConfiguration {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (
                SupportConfiguration::OnOutriggers {
                    extension: ext1, ..
                },
                SupportConfiguration::OnOutriggers {
                    extension: ext2, ..
                },
            ) => std::mem::discriminant(ext1) == std::mem::discriminant(ext2),
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl ConfigurationMatch for BoomConfiguration {
    fn matches(&self, other: &Self) -> bool {
        match (self.length_distance(), other.length_distance()) {
            (Ok(my_length), Ok(other_length)) => (my_length - other_length).abs().get::<foot>() < 0.01,
            _ => false,
        }
    }
}

impl ConfigurationMatch for CounterweightConfiguration {
    fn matches(&self, other: &Self) -> bool {
        match (self.to_uom_mass(), other.to_uom_mass()) {
            (Ok(my_weight), Ok(other_weight)) => (my_weight - other_weight).abs().get::<pound>() < 1.0,
            _ => false,
        }
    }
}

impl LoadChartPackage {
    /// Create new empty package
    pub fn new(crane_info: CraneInfo) -> Self {
        Self {
            crane_info,
            charts: Vec::new(),
        }
    }

    /// Add a chart to this package
    pub fn add_chart(&mut self, chart: LoadChart) {
        self.charts.push(chart);
    }

    /// Load from JSON file
    pub fn from_json_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let package = serde_json::from_str(&json)?;
        Ok(package)
    }

    /// Save to JSON file
    pub fn to_json_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Find the appropriate load chart for a given configuration
    pub fn find_chart(&self, config: &ChartConfiguration) -> Option<&LoadChart> {
        self.charts
            .iter()
            .find(|chart| chart.matches_configuration(config))
    }

    /// Get all charts for a specific support configuration
    pub fn charts_for_support(&self, support: &SupportConfiguration) -> Vec<&LoadChart> {
        self.charts
            .iter()
            .filter(|chart| chart.configuration.support.matches(support))
            .collect()
    }
}

impl Default for LoadChartPackage {
    fn default() -> Self {
        Self {
            crane_info: CraneInfo {
                manufacturer: "Unknown".into(),
                model: "Unknown".into(),
                serial_number: None,
                crane_type: CraneType::MobileTelescopic,
                year: None,
                chart_revision: None,
            },
            charts: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_chart_us() -> LoadChart {
        let mut capacity_data = CapacityData::new();

        capacity_data.boom_lengths = vec![LengthValue::new(154.2, "ft")];
        capacity_data.data = vec![vec![
            (
                LengthValue::new(20.0, "ft"),
                MassValue::new(242500.0, "lbs"),
            ),
            (
                LengthValue::new(40.0, "ft"),
                MassValue::new(152000.0, "lbs"),
            ),
            (
                LengthValue::new(60.0, "ft"),
                MassValue::new(97000.0, "lbs"),
            ),
        ]];

        LoadChart {
            id: "test_us".into(),
            description: "Test US units".into(),
            configuration: ChartConfiguration {
                support: SupportConfiguration::OnOutriggers {
                    extension: OutriggerExtension::Full,
                    swing_restriction: Some(SwingRestriction::Full360),
                },
                boom: BoomConfiguration {
                    length: LengthValue::new(154.2, "ft"),
                    angle_range: None,
                    jib: None,
                },
                counterweight: None,
                additional: HashMap::new(),
            },
            capacity_data,
            notes: Vec::new(),
        }
    }

    fn create_test_chart_metric() -> LoadChart {
        let mut capacity_data = CapacityData::new();

        // 47 m boom (roughly 154 ft)
        capacity_data.boom_lengths = vec![LengthValue::new(47.0, "m")];
        capacity_data.data = vec![vec![
            (LengthValue::new(6.0, "m"), MassValue::new(110000.0, "kg")), // ~20 ft, ~242k lbs
            (LengthValue::new(12.0, "m"), MassValue::new(69000.0, "kg")), // ~40 ft, ~152k lbs
            (LengthValue::new(18.0, "m"), MassValue::new(44000.0, "kg")), // ~60 ft, ~97k lbs
            (LengthValue::new(24.0, "m"), MassValue::new(31000.0, "kg")), // ~80 ft, ~68k lbs
            (LengthValue::new(30.0, "m"), MassValue::new(23000.0, "kg")), // ~100 ft, ~50k lbs
        ]];
        LoadChart {
            id: "test_metric".into(),
            description: "Test metric units".into(),
            configuration: ChartConfiguration {
                support: SupportConfiguration::OnOutriggers {
                    extension: OutriggerExtension::Full,
                    swing_restriction: Some(SwingRestriction::Full360),
                },
                boom: BoomConfiguration {
                    length: LengthValue::new(47.0, "m"),
                    angle_range: None,
                    jib: None,
                },
                counterweight: None,
                additional: HashMap::new(),
            },
            capacity_data,
            notes: Vec::new(),
        }
    }

    #[test]
    fn test_us_chart_exact_lookup() {
        let chart = create_test_chart_us();

        let capacity = chart
            .capacity_exact(Length::new::<foot>(154.2), Length::new::<foot>(40.0))
            .unwrap();

        assert_relative_eq!(capacity.get::<pound>(), 152000.0);
    }

    #[test]
    fn test_us_chart_interpolation() {
        let chart = create_test_chart_us();

        // Test at 30 ft (midpoint between 20 and 40)
        let capacity = chart
            .capacity_interpolated(Length::new::<foot>(154.2), Length::new::<foot>(30.0))
            .unwrap();

        // Should be ~197,250 lbs (midpoint between 242,500 and 152,000)
        assert_relative_eq!(capacity.get::<pound>(), 197250.0, epsilon = 1.0);
    }

    #[test]
    fn test_metric_chart_exact_lookup() {
        let chart = create_test_chart_metric();

        // Query in feet, but chart is in meters
        let capacity = chart
            .capacity_exact(Length::new::<meter>(47.0), Length::new::<meter>(12.0))
            .unwrap();

        assert_relative_eq!(capacity.get::<kilogram>(), 69000.0);
    }

    #[test]
    fn test_metric_chart_with_us_inputs() {
        let chart = create_test_chart_metric();

        // Query in feet, chart automatically converts
        let capacity = chart
            .capacity_exact(
                Length::new::<foot>(154.199), // ~47 meters
                Length::new::<foot>(39.3701),  // ~12 meters
            )
            .unwrap();

        // Should get ~69000 kg converted to pounds
        assert_relative_eq!(capacity.get::<pound>(), 152118.96, epsilon = 100.0);
    }

    #[test]
    fn test_unit_conversions() {
        let chart = create_test_chart_us();

        // Same query in different units should give same result
        let cap_ft = chart
            .capacity_exact(Length::new::<foot>(154.2), Length::new::<foot>(60.0))
            .unwrap();

        let cap_m = chart
            .capacity_exact(Length::new::<meter>(47.0), Length::new::<meter>(18.288))
            .unwrap();

        assert_relative_eq!(cap_ft.get::<pound>(), cap_m.get::<pound>(), epsilon = 100.0);
    }

    #[test]
    fn test_capacity_data_creation() {
        let mut data = CapacityData::new();

        data.boom_lengths = vec![LengthValue::new(100.0, "ft")];
        data.data = vec![vec![
            (
                LengthValue::new(20.0, "ft"),
                MassValue::new(50000.0, "lbs"),
            ),
            (
                LengthValue::new(40.0, "ft"),
                MassValue::new(30000.0, "lbs"),
            ),
        ]];

        let capacity = data
            .capacity_at(
                0,
                LengthValue::new(20.0, "ft").to_distance().unwrap(),
                Some(0.1),
            )
            .unwrap();

        assert_relative_eq!(capacity.unwrap().get::<pound>(), 50000.0);
    }

    #[test]
    fn test_boom_configuration_conversion() {
        let boom = BoomConfiguration {
            length: LengthValue::new(154.2, "ft"),
            angle_range: None,
            jib: None,
        };

        let distance = boom.length_distance().unwrap();
        assert_relative_eq!(distance.get::<foot>(), 154.2);

        // Convert to metric
        let boom_metric = BoomConfiguration {
            length: LengthValue::new(47.0, "m"),
            angle_range: None,
            jib: None,
        };

        let distance_metric = boom_metric.length_distance().unwrap();
        assert_relative_eq!(distance_metric.get::<meter>(), 47.0);
        assert_relative_eq!(distance_metric.get::<foot>(), 154.2, epsilon = 0.1);
    }

    #[test]
    fn test_counterweight_conversion() {
        let cw = CounterweightConfiguration {
            weight: MassValue::new(110200.0, "lbs"),
            configuration: "Standard".into(),
        };

        let weight = cw.to_uom_mass().unwrap();
        assert_relative_eq!(weight.get::<pound>(), 110200.0);

        let cw_metric = CounterweightConfiguration {
            weight: MassValue::new(50000.0, "kg"),
            configuration: "Standard".into(),
        };

        let weight_metric = cw_metric.to_uom_mass().unwrap();
        assert_relative_eq!(weight_metric.get::<kilogram>(), 50000.0);
        assert_relative_eq!(weight_metric.get::<pound>(), 110231.0, epsilon = 1.0);
    }
}

