//! Model extraction utilities.
#![allow(dead_code, clippy::too_many_arguments, clippy::needless_range_loop)]
//!
//! When Hayashi passes a model to a native plugin, it serializes it as a
//! `HayashiValue::Dict` with fields like `variable`, `coef`, `std_err`,
//! `p_value`, `r2`, `n`, etc. This module provides a typed wrapper.

use crate::helpers::val_as_f64;
use hayashi_plugin_sdk::value::HayashiValue;
use hayashi_plugin_sdk::arrow::array::{Array, ArrayRef};
use std::collections::HashMap;
/// Extracted model data for LaTeX generation.
pub struct ModelData {
    pub model_type: String,
    pub variables: Vec<String>,
    pub coef: Vec<f64>,
    pub std_err: Vec<f64>,
    pub stat: Vec<f64>,      // t or z values
    pub p_value: Vec<f64>,
    pub conf_low: Vec<f64>,
    pub conf_high: Vec<f64>,
    pub fit: HashMap<String, f64>,
}

impl ModelData {
    /// Extract model data from a HayashiValue (expected: Dict from model serialization).
    pub fn from_value(val: &HayashiValue) -> Result<Self, String> {
        let map = match val {
            HayashiValue::Dict(d) => d,
            other => return Err(format!(
                "expected model (dict), got {}",
                other.type_name()
            )),
        };

        let model_type = match map.get("__model_type__") {
            Some(HayashiValue::Str(s)) => s.clone(),
            _ => "unknown".to_string(),
        };

        let variables = extract_str_list(map, "variable");
        let coef = extract_f64_list(map, "coef");
        let std_err = extract_f64_list(map, "std_err");
        let _stat = extract_f64_list(map, "t")
            .into_iter()
            .chain(extract_f64_list(map, "z"))
            .collect::<Vec<_>>();
        // Prefer t, fallback to z
        let stat = if !extract_f64_list(map, "t").is_empty() {
            extract_f64_list(map, "t")
        } else {
            extract_f64_list(map, "z")
        };
        let p_value = extract_f64_list(map, "p_value");
        let conf_low = extract_f64_list(map, "conf_low");
        let conf_high = extract_f64_list(map, "conf_high");

        // Extract all scalar fit statistics
        let mut fit = HashMap::new();
        for (k, v) in map.iter() {
            if k == "__model_type__" || k == "variable" || k == "coef"
                || k == "std_err" || k == "t" || k == "z" || k == "p_value"
                || k == "conf_low" || k == "conf_high" {
                continue;
            }
            if let Some(f) = val_as_f64(v) {
                fit.insert(k.clone(), f);
            }
        }

        Ok(ModelData {
            model_type,
            variables,
            coef,
            std_err,
            stat,
            p_value,
            conf_low,
            conf_high,
            fit,
        })
    }

    /// Number of coefficients.
    pub fn n_coef(&self) -> usize {
        self.coef.len()
    }

    /// Find index of a variable by name (handles _cons, const, intercept).
    pub fn find_var(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|v| {
            v == name
                || (name == "_cons" && (v == "const" || v == "intercept"))
                || (name == "const" && (v == "_cons" || v == "intercept"))
        })
    }

    /// Check if model has a constant term.
    pub fn has_constant(&self) -> Option<usize> {
        self.find_var("_cons")
            .or_else(|| self.find_var("const"))
            .or_else(|| self.find_var("intercept"))
    }

    /// Get N (observations).
    pub fn n_obs(&self) -> Option<f64> {
        self.fit.get("n").copied()
    }

    /// Get R² (handles r2, pseudo_r2).
    pub fn r_squared(&self) -> Option<f64> {
        self.fit.get("r2").copied().or_else(|| self.fit.get("pseudo_r2").copied())
    }

    /// Get a fit statistic by name.
    pub fn get_stat(&self, name: &str) -> Option<f64> {
        self.fit.get(name).copied()
    }
}

fn extract_str_list(map: &HashMap<String, HayashiValue>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(HayashiValue::List(lst)) => {
            lst.iter()
                .map(|v| match v {
                    HayashiValue::Str(s) => s.clone(),
                    _ => format!("{:?}", v),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn extract_f64_list(map: &HashMap<String, HayashiValue>, key: &str) -> Vec<f64> {
    match map.get(key) {
        Some(HayashiValue::List(lst)) => {
            lst.iter()
                .map(|v| val_as_f64(v).unwrap_or(f64::NAN))
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Extract a DataFrame-like dict into column names and a map of column → Vec<f64>.
/// Handles both Dict-of-Lists (JSON serialization) and Arrow FFI.
pub struct DfData {
    pub columns: Vec<String>,
    pub data: HashMap<String, Vec<f64>>,
    pub data_str: HashMap<String, Vec<String>>,
}

impl DfData {
    /// Extract from a HayashiValue that represents a DataFrame.
    /// Handles both Dict-of-Lists (JSON serialization) and Arrow FFI.
    pub fn from_value(val: &HayashiValue) -> Result<Self, String> {
        match val {
            HayashiValue::Dict(d) => Self::from_dict(d),
            HayashiValue::Arrow(array_ptr, schema_ptr) => {
                // Import the Arrow array
                let array_ref = <ArrayRef as hayashi_plugin_sdk::value::FromHayashi>::from_hayashi(
                    HayashiValue::Arrow(*array_ptr, *schema_ptr)
                ).map_err(|e| format!("failed to import Arrow array: {e}"))?;
                Self::from_arrow(&array_ref)
            }
            other => Err(format!(
                "expected dataframe (dict or arrow_array), got {}",
                other.type_name()
            )),
        }
    }

    fn from_dict(map: &HashMap<String, HayashiValue>) -> Result<Self, String> {
        let mut columns = Vec::new();
        let mut data = HashMap::new();
        let mut data_str = HashMap::new();

        for (k, v) in map.iter() {
            // Skip arrow FFI pointers
            if k == "__arrow_array_ptr__" || k == "__arrow_schema_ptr__" {
                continue;
            }
            columns.push(k.clone());
            if let HayashiValue::List(lst) = v {
                let all_numeric = lst.iter().all(|item| {
                    matches!(item, HayashiValue::Float(_) | HayashiValue::Int(_) | HayashiValue::Nil)
                });
                if all_numeric {
                    let floats: Vec<f64> = lst.iter()
                        .map(|item| val_as_f64(item).unwrap_or(f64::NAN))
                        .collect();
                    data.insert(k.clone(), floats);
                } else {
                    let strings: Vec<String> = lst.iter()
                        .map(|item| match item {
                            HayashiValue::Str(s) => s.clone(),
                            HayashiValue::Float(f) => crate::helpers::fmt_trim(*f, 4),
                            HayashiValue::Int(i) => i.to_string(),
                            HayashiValue::Nil => "".to_string(),
                            _ => format!("{:?}", item),
                        })
                        .collect();
                    data_str.insert(k.clone(), strings);
                }
            }
        }
        columns.sort();
        Ok(DfData { columns, data, data_str })
    }

    fn from_arrow(array: &ArrayRef) -> Result<Self, String> {
        use hayashi_plugin_sdk::arrow::datatypes::DataType;
        use hayashi_plugin_sdk::arrow::array::StructArray;

        // DataFrame comes as a StructArray
        let struct_array = array.as_any().downcast_ref::<StructArray>()
            .ok_or_else(|| "expected StructArray from DataFrame".to_string())?;

        let fields = struct_array.fields();
        let mut columns = Vec::new();
        let mut data = HashMap::new();
        let mut data_str = HashMap::new();

        for (i, field) in fields.iter().enumerate() {
            let col_name = field.name().to_string();
            let col_array = struct_array.column(i);
            columns.push(col_name.clone());

            match col_array.data_type() {
                DataType::Float64 => {
                    let arr = col_array.as_any().downcast_ref::<hayashi_plugin_sdk::arrow::array::Float64Array>()
                        .ok_or_else(|| "failed to downcast Float64Array".to_string())?;
                    let vals: Vec<f64> = (0..arr.len())
                        .map(|i| if arr.is_null(i) { f64::NAN } else { arr.value(i) })
                        .collect();
                    data.insert(col_name, vals);
                }
                DataType::Int64 => {
                    let arr = col_array.as_any().downcast_ref::<hayashi_plugin_sdk::arrow::array::Int64Array>()
                        .ok_or_else(|| "failed to downcast Int64Array".to_string())?;
                    let vals: Vec<f64> = (0..arr.len())
                        .map(|i| if arr.is_null(i) { f64::NAN } else { arr.value(i) as f64 })
                        .collect();
                    data.insert(col_name, vals);
                }
                DataType::Boolean => {
                    let arr = col_array.as_any().downcast_ref::<hayashi_plugin_sdk::arrow::array::BooleanArray>()
                        .ok_or_else(|| "failed to downcast BooleanArray".to_string())?;
                    let vals: Vec<f64> = (0..arr.len())
                        .map(|i| if arr.is_null(i) { f64::NAN } else { if arr.value(i) { 1.0 } else { 0.0 } })
                        .collect();
                    data.insert(col_name, vals);
                }
                DataType::Utf8 => {
                    let arr = col_array.as_any().downcast_ref::<hayashi_plugin_sdk::arrow::array::StringArray>()
                        .ok_or_else(|| "failed to downcast StringArray".to_string())?;
                    let vals: Vec<String> = (0..arr.len())
                        .map(|i| if arr.is_null(i) { String::new() } else { arr.value(i).to_string() })
                        .collect();
                    data_str.insert(col_name, vals);
                }
                dt => {
                    // Fallback: try to convert to f64
                    let vals: Vec<f64> = (0..col_array.len()).map(|_| f64::NAN).collect();
                    data.insert(col_name, vals);
                    let _ = dt;
                }
            }
        }
        columns.sort();
        Ok(DfData { columns, data, data_str })
    }

    /// Get a numeric column.
    pub fn get_col(&self, name: &str) -> Result<&Vec<f64>, String> {
        self.data.get(name)
            .ok_or_else(|| format!("column '{}' not found or not numeric", name))
    }

    /// Get a string column.
    pub fn get_str_col(&self, name: &str) -> Option<&Vec<String>> {
        self.data_str.get(name)
    }

    /// Number of rows (from first column).
    pub fn nrow(&self) -> usize {
        self.data.values().next().map(|v| v.len())
            .or_else(|| self.data_str.values().next().map(|v| v.len()))
            .unwrap_or(0)
    }
}
