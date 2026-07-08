//! Internal helper functions for formatting numbers and strings in LaTeX.

use hayashi_plugin_sdk::value::HayashiValue;
use std::collections::HashMap;

/// Format a float with a given number of decimal places, trimming trailing zeros.
pub fn fmt_trim(x: f64, decimals: usize) -> String {
    if x.is_nan() {
        return "---".to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let s = format!("{:.*}", decimals, x);
    // Trim trailing zeros after decimal point
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() || trimmed == "-" {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        s
    }
}

/// Format a float with fixed decimal places (keeps trailing zeros).
pub fn fmt_fixed(x: f64, decimals: usize) -> String {
    if x.is_nan() {
        return "---".to_string();
    }
    format!("{:.*}", decimals, x)
}

/// Significance stars from p-value.
pub fn stars(p: f64) -> &'static str {
    if p < 0.01 {
        "***"
    } else if p < 0.05 {
        "**"
    } else if p < 0.10 {
        "*"
    } else {
        ""
    }
}

/// Escape special LaTeX characters in a string.
pub fn esc(s: &str) -> String {
    s.replace('&', "\\&")
        .replace('%', "\\%")
        .replace('_', "\\_")
        .replace('#', "\\#")
}

/// Repeat a string n times.
pub fn repeat(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Join a slice of strings with a separator.
pub fn join(lst: &[String], sep: &str) -> String {
    lst.join(sep)
}

/// Extract a float from a HayashiValue (handles Float, Int, Nil).
pub fn val_as_f64(v: &HayashiValue) -> Option<f64> {
    match v {
        HayashiValue::Float(f) => Some(*f),
        HayashiValue::Int(i) => Some(*i as f64),
        HayashiValue::Nil => None,
        _ => None,
    }
}

/// Extract a string from a HayashiValue.
pub fn val_as_string(v: &HayashiValue) -> String {
    match v {
        HayashiValue::Str(s) => s.clone(),
        HayashiValue::Float(f) => fmt_trim(*f, 4),
        HayashiValue::Int(i) => i.to_string(),
        HayashiValue::Bool(b) => b.to_string(),
        _ => format!("{:?}", v),
    }
}

/// Get an optional string field from a dict, with default.
pub fn opt_str(opts: &HashMap<String, HayashiValue>, key: &str, default: &str) -> String {
    match opts.get(key) {
        Some(HayashiValue::Str(s)) => s.clone(),
        _ => default.to_string(),
    }
}

/// Get an optional integer field from a dict, with default.
pub fn opt_int(opts: &HashMap<String, HayashiValue>, key: &str, default: i64) -> i64 {
    match opts.get(key) {
        Some(HayashiValue::Int(i)) => *i,
        Some(HayashiValue::Float(f)) => *f as i64,
        _ => default,
    }
}

/// Get an optional bool field from a dict, with default.
pub fn opt_bool(opts: &HashMap<String, HayashiValue>, key: &str, default: bool) -> bool {
    match opts.get(key) {
        Some(HayashiValue::Bool(b)) => *b,
        Some(HayashiValue::Int(i)) => *i != 0,
        _ => default,
    }
}

/// Get an optional float field from a dict, with default.
pub fn opt_f64(opts: &HashMap<String, HayashiValue>, key: &str, default: f64) -> f64 {
    match opts.get(key) {
        Some(HayashiValue::Float(f)) => *f,
        Some(HayashiValue::Int(i)) => *i as f64,
        _ => default,
    }
}

/// Get a list of strings from a dict field.
pub fn opt_str_list(opts: &HashMap<String, HayashiValue>, key: &str) -> Vec<String> {
    match opts.get(key) {
        Some(HayashiValue::List(lst)) => lst.iter().map(val_as_string).collect(),
        _ => Vec::new(),
    }
}

/// Get a dict of variable labels from opts.
pub fn opt_labels(opts: &HashMap<String, HayashiValue>, key: &str) -> HashMap<String, String> {
    match opts.get(key) {
        Some(HayashiValue::Dict(d)) => {
            d.iter()
                .filter_map(|(k, v)| {
                    if let HayashiValue::Str(s) = v {
                        Some((k.clone(), s.clone()))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => HashMap::new(),
    }
}

/// Wrap content in a table environment with optional caption and label.
pub fn wrap_table(content: &str, caption: &str, label: &str) -> String {
    let mut s = String::new();
    if !caption.is_empty() {
        s.push_str("\\begin{table}[htbp]\n\\centering\n");
        s.push_str(&format!("\\caption{{{}}}\n", esc(caption)));
        if !label.is_empty() {
            s.push_str(&format!("\\label{{{}}}\n", label));
        }
    }
    s.push_str(content);
    if !caption.is_empty() {
        s.push_str("\\end{table}\n");
    }
    s
}

/// Star legend footer.
pub fn star_legend() -> &'static str {
    "\\par\\vspace{2pt}\n\\footnotesize \\*** p<0.01, ** p<0.05, * p<0.1\n"
}
