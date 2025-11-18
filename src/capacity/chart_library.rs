use crate::capacity::load_chart::*;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Error types for chart library operations
#[derive(Debug, thiserror::Error)]
pub enum ChartLibraryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Chart package not found for crane: {0} {1}")]
    PackageNotFound(String, String),

    #[error("No matching chart found for configuration")]
    NoMatchingChart,

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
}


/// Library of load chart packagees for multiple cranes
#[derive(Debug, Default)]
pub struct ChartLibrary {
    /// Maps "Manufacturer:Model" -> LoadChartPackage
    packages: HashMap<String, LoadChartPackage>,

    /// Base directory where chart files are stored
    base_path: Option<PathBuf>,
}

impl ChartLibrary {
    /// Create a new empty chart library
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            base_path: None,
        }
    }

    /// Create a chart library and load all charts from a directory
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self, ChartLibraryError> {
        let mut library = Self::new();
        library.base_path = Some(path.as_ref().to_path_buf());
        library.load_all_from_directory(path)?;
        Ok(library)
    }

    /// Load all JSON chart files from a directory
    pub fn load_all_from_directory(&mut self, path: impl AsRef<Path>) -> Result<(), ChartLibraryError> {
        let dir = fs::read_dir(path)?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_package_from_file(&path) {
                    Ok(_) => println!("Loaded: {}", path.display()),
                    Err(e) => eprintln!("Skipped {}: {}", path.display(), e),
                }
            }
        }
        Ok(())
    }



    /// Load a chart package from a JSON file
    pub fn load_package_from_file(&mut self, path: impl AsRef<Path>) -> Result<(), ChartLibraryError> {
        let json = fs::read_to_string(path.as_ref())?;
        let package: LoadChartPackage = serde_json::from_str(&json)?;

        let key = format!("{}:{}", package.crane_info.manufacturer, package.crane_info.model);
        self.packages.insert(key, package);
        
        Ok(())
    }

    /// Add a chart package directly
    pub fn add_package(&mut self, package: LoadChartPackage) {
        let key = format!("{}:{}", package.crane_info.manufacturer, package.crane_info.model);
        self.packages.insert(key, package);
    }

    /// Get a chart package by manufacturer and model
    pub fn get_package(&self, manufacturer: &str, model: &str) -> Option<&LoadChartPackage> {
        let key = format!("{}:{}", manufacturer, model);
        self.packages.get(&key)
    }

    /// Get a mutable chart package
    pub fn get_package_mut(&mut self, manufacturer: &str, model: &str) -> Option<&mut LoadChartPackage> {
        let key = format!("{}:{}", manufacturer, model);
        self.packages.get_mut(&key)
    }

    /// Find the best matching chart for a configuration
    pub fn find_chart(
        &self,
        manufacturer: &str,
        model: &str,
        config: &ChartConfiguration,
    ) -> Result<&LoadChart, ChartLibraryError> {
        let package = self.get_package(manufacturer, model)
            .ok_or_else(|| ChartLibraryError::PackageNotFound(
                manufacturer.to_string(),
                model.to_string(),
            ))?;

        package.find_chart(config)
                .ok_or(ChartLibraryError::NoMatchingChart)
        }

        /// Get all available manufacturers
        pub fn manufacturers(&self) -> Vec<String> {
            let mut manufacturers: Vec<String> = self.packages
            .values()
            .map(|p| p.crane_info.manufacturer.clone())
            .collect();

        manufacturers.sort();
        manufacturers.dedup();
        manufacturers
    }

    /// Get all models for a manufacturer
    pub fn models(&self, manufacturer: &str) -> Vec<String> {
        self.packages
            .values()
            .filter(|p| p.crane_info.manufacturer == manufacturer)
            .map(|p| p.crane_info.model.clone())
            .collect()
    }

    /// Count total number of charts across all packages
    pub fn total_charts(&self) -> usize {
        self.packages.values()
            .map(|p| p.charts.len())
            .sum()
    }

    /// Validate all charts in the library
    pub fn validate_all(&self) -> Result<ValidationReport, ChartLibraryError> {
        let mut report = ValidationReport::new();

        for (key, package) in &self.packages {
            for chart in &package.charts {
                if let Err(errors) = validate_chart(chart) {
                    report.add_errors(key, &chart.id, errors);
                }
            }
        }

        Ok(report)
    }

    /// Remove a package from the library
    pub fn remove_package(&mut self, manufacturer: &str, model: &str) -> Option<LoadChartPackage> {
        let key = format!("{}:{}", manufacturer, model);
        self.packages.remove(&key)
    }

    /// Clear all packages
    pub fn clear(&mut self) {
        self.packages.clear();
    }

    /// Check if library is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Get number of packages
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

/// Validation report for charts
#[derive(Debug, Default)]
pub struct ValidationReport {
    errors: HashMap<String, Vec<ChartError>>,
}

#[derive(Debug)]
pub struct ChartError {
    pub chart_id: String,
    pub error: String,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_errors(&mut self, package_key: &str, chart_id: &str, errors: Vec<String>) {
        let entry = self.errors.entry(package_key.to_string()).or_insert_with(Vec::new);
        for error in errors {
            entry.push(ChartError {
                chart_id: chart_id.to_string(),
                error,
            });
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    pub fn print_report(&self) {
        if self.is_valid() {
            println!("All charts valid");
            return;
        }

        println!("Validation errors found: ");
        for (package, errors) in &self.errors {
            println!("\n{}", package);
            for error in errors {
                println!(" - [{}] {}", error.chart_id, error.error);
            }
        }
    }
}

/// Validate a single chart
fn validate_chart(chart: &LoadChart) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    
    // Check boom lengths have valid units
    for (i, boom) in chart.capacity_data.boom_lengths.iter().enumerate() {
        if let Err(e) = boom.to_distance() {
            errors.push(format!("Boom length {}: {}", i , e));
        }
    }

    // Check capacity data has valid units
    for (boom_idx, row) in chart.capacity_data.data.iter().enumerate() {
        for (point_idx, (radius, capacity)) in row.iter().enumerate() {
            if let Err(e) = radius.to_distance() {
                errors.push(format!("Boom {} point {}: invalid radius unit - {}", boom_idx, point_idx, e));
            }
            if let Err(e) = capacity.to_mass() {
                errors.push(format!("Boom {} point {}: invalid capacity unit - {}", boom_idx, point_idx, e));
            }
        }
    }

    // Check boom configuration
    if let Err(e) = chart.configuration.boom.length_distance() {
        errors.push(format!("Boom configuration: {}", e));
    }

    // Check counterweight if present
    if let Some(ref cw) = chart.configuration.counterweight {
        if let Err(e) = cw.to_uom_mass() {
            errors.push(format!("Counterweight configuration: {}", e));
        }
    }

    // Check data consistency
    if chart.capacity_data.boom_lengths.len() != chart.capacity_data.data.len() {
        errors.push(format!(
            "Mismatch: {} boom lengths but {} data rows",
            chart.capacity_data.boom_lengths.len(),
            chart.capacity_data.data.len()
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}


#[cfg(test)]
mod tests {
    use crate::equipment::CraneType;
    use super::*;
    use crate::types::*;

    fn create_test_package() -> LoadChartPackage {
        let crane_info = CraneInfo {
            manufacturer: "Grove".into(),
            model: "GMK5250L".into(),
            serial_number: None,
            crane_type: CraneType::AllTerrain,
            year: Some(2020),
            chart_revision: Some("Rev 2020-03".into()),
        };

        let mut capacity_data = CapacityData::new();
        capacity_data.boom_lengths = vec![
            LengthValue::new(154.2, "ft"),
        ];
        capacity_data.data = vec![
            vec![
            (LengthValue::new(20.0, "ft"), MassValue::new(242500.0, "lbs")),
            (LengthValue::new(40.0, "ft"), MassValue::new(152000.0, "lbs")),
            ],
        ];

        let chart = LoadChart {
            id: "test_chart".into(),
            description: "Test chart".into(),
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
                counterweight: Some(CounterweightConfiguration {
                    weight: MassValue::new(110200.0, "lbs"),
                    configuration: "Standard".into(),
                }),
                additional: std::collections::HashMap::new(),
            },
            capacity_data,
            notes: vec![],
        };

        LoadChartPackage {
            crane_info,
            charts: vec![chart],
        }
    }

    #[test]
    fn test_library_add_and_get() {
        let mut library = ChartLibrary::new();
        let package = create_test_package();

        library.add_package(package);

        let retrieved = library.get_package("Grove", "GMK5250L");
        assert!(retrieved.is_some());

        let retrieved = library.get_package("Liebherr", "LTM1250");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_library_manufacturers() {
        let mut library = ChartLibrary::new();
        library.add_package(create_test_package());

        let manufacturers = library.manufacturers();
        assert_eq!(manufacturers, vec!["Grove"]);
    }

    #[test]
    fn test_chart_validation() {
        let package = create_test_package();
        let chart = &package.charts[0];

        let result = validate_chart(chart);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_units() {
        let mut chart = create_test_package().charts.into_iter().next().unwrap();
    
        // Add invalid unit
        chart.capacity_data.boom_lengths.push(
            LengthValue::new(200.0, "invalid_unit")
        );

        let result = validate_chart(&chart);
        assert!(result.is_err());
    }

    #[test]
    fn test_library_count() {
        let mut library = ChartLibrary::new();
        assert!(library.is_empty());
        assert_eq!(library.package_count(), 0);

        library.add_package(create_test_package());
        assert!(!library.is_empty());
        assert_eq!(library.package_count(), 1);
        assert_eq!(library.total_charts(), 1);
    }
}
