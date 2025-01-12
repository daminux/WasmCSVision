use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use csv::{ReaderBuilder, Trim};
use regex::Regex;
use lazy_static::lazy_static;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

lazy_static! {
    static ref DATE_PATTERNS: [Regex; 3] = [
        Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),
        Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap(),
        Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap()
    ];
    static ref DATETIME_PATTERN: Regex = 
        Regex::new(r"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}(:\d{2})?(\.\d+)?(Z|[+-]\d{2}:?\d{2})?$").unwrap();
    static ref TIME_PATTERN: Regex = 
        Regex::new(r"^\d{2}:\d{2}(:\d{2})?(\.\d+)?$").unwrap();
    static ref EMAIL_PATTERN: Regex = 
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref URL_PATTERN: Regex = 
        Regex::new(r"^(https?://)?[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/[^\s]*)?$").unwrap();
    static ref IPV4_PATTERN: Regex = 
        Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$").unwrap();
    static ref IPV6_PATTERN: Regex = 
        Regex::new(r"^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$").unwrap();
}

#[derive(Serialize, Deserialize)]
struct Column {
    name: String,
    type_name: String,
    type_details: TypeDetails,
    unique_values: usize,
    null_count: usize,
    min_value: Option<String>,
    max_value: Option<String>,
    min_length: usize,
    max_length: usize,
    sample_values: Vec<String>,
    valid_count: usize,
    total_count: usize,
    analyzed_count: usize,
}

#[derive(Serialize, Deserialize)]
struct TypeDetails {
    subtypes: Vec<String>,
    confidence: f64,
    format_examples: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Analysis {
    row_count: usize,
    column_count: usize,
    columns: Vec<Column>,
    detected_delimiter: char,
    sample_size: Option<usize>,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct AnalyzerConfig {
    sample_size: Option<usize>,
}

#[wasm_bindgen]
impl AnalyzerConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(setter)]
    pub fn set_sample_size(&mut self, size: Option<usize>) {
        self.sample_size = size;
    }
}

#[wasm_bindgen]
pub struct CSVAnalyzer {
    config: AnalyzerConfig,
}

#[wasm_bindgen]
impl CSVAnalyzer {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<AnalyzerConfig>) -> Self {
        console_log!("CSVAnalyzer::new()");
        CSVAnalyzer {
            config: config.unwrap_or_default(),
        }
    }

    fn get_sample_size(&self, total_size: usize) -> usize {
        match self.config.sample_size {
            Some(size) => std::cmp::min(size, total_size),
            None => total_size,
        }
    }

    fn detect_delimiter(content: &str) -> char {
        let common_delimiters = [',', ';', '\t', '|'];
        let first_line = content.lines().next().unwrap_or("");
        
        common_delimiters
            .iter()
            .max_by_key(|&&delimiter| first_line.matches(delimiter).count())
            .map(|&delimiter| delimiter)
            .unwrap_or(',')
    }

    fn is_integer(value: &str) -> bool {
        value.parse::<i64>().is_ok()
    }

    fn is_float(value: &str) -> bool {
        let normalized = value.replace(',', ".");
        normalized.parse::<f64>().is_ok()
    }

    fn is_boolean(value: &str) -> bool {
        let lower_value = value.to_lowercase();
        matches!(lower_value.as_str(), "true" | "false" | "1" | "0" | "yes" | "no" | "oui" | "non")
    }

    fn detect_column_type(&self, values: &[&str]) -> (String, TypeDetails, usize) {
        let mut counts = HashMap::new();
        let mut format_examples = Vec::new();
        let mut total_valid = 0;

        let sample_size = self.get_sample_size(values.len());
        console_log!("Analyzing {} values out of {}", sample_size, values.len());

        for value in values.iter().take(sample_size) {
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            
            total_valid += 1;
            
            let detected_type = if Self::is_boolean(value) {
                "boolean"
            } else if Self::is_integer(value) {
                "integer"
            } else if Self::is_float(value) {
                "float"
            } else if DATE_PATTERNS.iter().any(|pattern| pattern.is_match(value)) {
                "date"
            } else if DATETIME_PATTERN.is_match(value) {
                "datetime"
            } else if TIME_PATTERN.is_match(value) {
                "time"
            } else if EMAIL_PATTERN.is_match(value) {
                "email"
            } else if URL_PATTERN.is_match(value) {
                "url"
            } else if IPV4_PATTERN.is_match(value) || IPV6_PATTERN.is_match(value) {
                "ip"
            } else {
                "string"
            };

            *counts.entry(detected_type).or_insert(0) += 1;

            if format_examples.len() < 3 && !format_examples.contains(&value.to_string()) {
                format_examples.push(value.to_string());
            }
        }

        if total_valid == 0 {
            return ("null".to_string(), TypeDetails {
                subtypes: vec!["null".to_string()],
                confidence: 1.0,
                format_examples: vec![],
            }, sample_size);
        }

        let (primary_type, primary_count) = counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&t, &c)| (t, c))
            .unwrap_or(("string", 0));

        let confidence = primary_count as f64 / total_valid as f64;
        
        let threshold = (total_valid as f64) * 0.05;
        let subtypes: Vec<String> = counts.iter()
            .filter(|(_, &count)| count as f64 >= threshold)
            .map(|(&t, _)| t.to_string())
            .collect();

        (primary_type.to_string(), TypeDetails {
            subtypes,
            confidence,
            format_examples,
        }, sample_size)
    }

    fn find_min_max(&self, values: &[&str], type_name: &str) -> (Option<String>, Option<String>, usize) {
        let sample_size = self.get_sample_size(values.len());
        let sampled_values: Vec<&str> = values.iter()
            .take(sample_size)
            .map(|&s| s)
            .collect();

        let result = match type_name {
            "integer" | "float" => {
                let numbers: Vec<f64> = sampled_values.iter()
                    .filter_map(|v| {
                        let v = v.trim().replace(',', ".");
                        v.parse::<f64>().ok()
                    })
                    .collect();

                let min = numbers.iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|v| v.to_string());
                let max = numbers.iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|v| v.to_string());
                (min, max)
            },
            "date" | "datetime" | "time" | "string" => {
                let valid_values: Vec<&str> = sampled_values.iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                let min = valid_values.iter().min().map(|&s| s.to_string());
                let max = valid_values.iter().max().map(|&s| s.to_string());
                (min, max)
            },
            _ => (None, None)
        };

        (result.0, result.1, sample_size)
    }

    fn find_length_stats(&self, values: &[&str]) -> (usize, usize, usize) {
        let sample_size = self.get_sample_size(values.len());
        let sampled_values: Vec<&str> = values.iter()
            .take(sample_size)
            .map(|&s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if sampled_values.is_empty() {
            return (0, 0, sample_size);
        }

        let min_length = sampled_values.iter()
            .map(|s| s.len())
            .min()
            .unwrap_or(0);

        let max_length = sampled_values.iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        (min_length, max_length, sample_size)
    }

    #[wasm_bindgen]
    pub fn analyze(&self, content: &str) -> Result<JsValue, JsValue> {
        console_log!("Starting analysis...");

        let delimiter = Self::detect_delimiter(content);
        console_log!("Detected delimiter: {}", delimiter);

        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter as u8)
            .trim(Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers: Vec<String> = match reader.headers() {
            Ok(headers) => headers.iter().map(|h| h.to_string()).collect(),
            Err(e) => return Err(JsValue::from_str(&format!("Error reading headers: {}", e))),
        };

        console_log!("Found {} columns", headers.len());

        let mut column_values: HashMap<String, Vec<String>> = HashMap::new();
        for header in &headers {
            column_values.insert(header.clone(), Vec::new());
        }

        for result in reader.records() {
            match result {
                Ok(record) => {
                    for (i, value) in record.iter().enumerate() {
                        if i < headers.len() {
                            if let Some(column) = column_values.get_mut(&headers[i]) {
                                column.push(value.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    console_log!("Warning: Error reading record: {}", e);
                    continue;
                }
            }
        }

        let mut analysis_columns = Vec::new();
        let row_count = column_values.values().next().map_or(0, |v| v.len());
        
        for header in headers.iter() {
            if let Some(values) = column_values.get(header) {
                let values_refs: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
                
                let unique_values: std::collections::HashSet<_> = values_refs.iter().collect();
                let null_count = values_refs.iter().filter(|v| v.trim().is_empty()).count();
                
                let (type_name, type_details, type_analyzed) = self.detect_column_type(&values_refs);
                let (min_value, max_value, minmax_analyzed) = self.find_min_max(&values_refs, &type_name);
                let (min_length, max_length, length_analyzed) = self.find_length_stats(&values_refs);

                let sample_values: Vec<String> = values_refs.iter()
                    .filter(|v| !v.trim().is_empty())
                    .take(5)
                    .map(|&s| s.to_string())
                    .collect();

                let total_count = values.len();
                let valid_count = total_count - null_count;
                let analyzed_count = std::cmp::min(type_analyzed, std::cmp::min(minmax_analyzed, length_analyzed));

                analysis_columns.push(Column {
                    name: header.clone(),
                    type_name,
                    type_details,
                    unique_values: unique_values.len(),
                    null_count,
                    min_value,
                    max_value,
                    min_length,
                    max_length,
                    sample_values,
                    valid_count,
                    total_count,
                    analyzed_count,
                });
            }
        }

        let analysis = Analysis {
            row_count,
            column_count: headers.len(),
            columns: analysis_columns,
            detected_delimiter: delimiter,
            sample_size: self.config.sample_size,
        };

        console_log!("Analysis complete");

        match serde_wasm_bindgen::to_value(&analysis) {
            Ok(value) => {
                console_log!("Successfully converted to JsValue");
                Ok(value)
            },
            Err(err) => {
                console_log!("Error converting to JsValue: {}", err.to_string());
                Err(JsValue::from_str(&format!("Serialization error: {}", err)))
            }
        }
    }
}