#![allow(clippy::not_unsafe_ptr_arg_deref)]
//! # haytex — LaTeX snippet generator for Hayashi
//!
//! Generates publication-ready LaTeX tables and equations from Hayashi
//! models, DataFrames, matrices, and test results.
//!
//! All functions return `String` containing LaTeX code (no `\documentclass`,
//! no preamble — just snippets ready to paste into a `.tex` file).
//!
//! ## Usage
//!
//! ```text
//! import("sheep-farm/haytex", as=haytex)
//! let m = ols(price ~ mpg + weight, auto)
//! let tex = haytex::regression([m], {"title": "Results"})
//! print(tex)
//! ```

use hayashi_plugin_sdk::hayashi_plugin;

hayashi_plugin!();

mod equations;
mod format;
mod helpers;
mod models;
mod tables;

// Re-export the main functions
pub use equations::{equation, forecast, margins, matrix};
pub use tables::{anova, codebook, correlation, diagnostics, regression, summary, table, tests};

#[cfg(test)]
mod tests {
    use crate::helpers::*;
    use hayashi_plugin_sdk::value::HayashiValue;
    use std::collections::HashMap;

    // ── helpers: fmt_trim ─────────────────────────────────────────────────────

    #[test]
    fn test_fmt_trim_zeros() {
        assert_eq!(fmt_trim(1.5000, 4), "1.5");
        assert_eq!(fmt_trim(0.0, 3), "0");
        assert_eq!(fmt_trim(f64::NAN, 3), "---");
    }

    #[test]
    fn test_fmt_trim_negatives() {
        assert_eq!(fmt_trim(-3.14, 2), "-3.14");
        assert_eq!(fmt_trim(-0.1, 3), "-0.1");
    }

    // ── helpers: fmt_fixed ────────────────────────────────────────────────────

    #[test]
    fn test_fmt_fixed_keeps_zeros() {
        assert_eq!(fmt_fixed(1.5, 4), "1.5000");
        assert_eq!(fmt_fixed(0.0, 2), "0.00");
        assert_eq!(fmt_fixed(f64::NAN, 2), "---");
    }

    // ── helpers: stars ────────────────────────────────────────────────────────

    #[test]
    fn test_stars() {
        assert_eq!(stars(0.001), "***");
        assert_eq!(stars(0.01), "**");
        assert_eq!(stars(0.05), "*");
        // 0.06 < 0.10 → ainda recebe uma estrela
        assert_eq!(stars(0.06), "*");
        assert_eq!(stars(0.5), "");
    }

    #[test]
    fn test_stars_boundaries() {
        // < 0.01 → ***
        assert_eq!(stars(0.0099), "***");
        // == 0.01 → **
        assert_eq!(stars(0.01), "**");
        // < 0.05 → **
        assert_eq!(stars(0.049), "**");
        // == 0.05 → *
        assert_eq!(stars(0.05), "*");
        // < 0.10 → *
        assert_eq!(stars(0.099), "*");
        // == 0.10 → ""
        assert_eq!(stars(0.10), "");
    }

    // ── helpers: esc ─────────────────────────────────────────────────────────

    #[test]
    fn test_esc_special_chars() {
        assert_eq!(esc("a & b"), "a \\& b");
        assert_eq!(esc("50%"), "50\\%");
        assert_eq!(esc("a_b"), "a\\_b");
        assert_eq!(esc("a#b"), "a\\#b");
        assert_eq!(esc("plain"), "plain");
    }

    // ── helpers: repeat / join ────────────────────────────────────────────────

    #[test]
    fn test_repeat() {
        assert_eq!(repeat("r", 3), "rrr");
        assert_eq!(repeat("x", 0), "");
    }

    #[test]
    fn test_join() {
        let v: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(join(&v, " & "), "a & b & c");
        assert_eq!(join(&v, ""), "abc");
    }

    // ── helpers: opt_str / opt_int / opt_bool / opt_f64 ──────────────────────

    #[test]
    fn test_opt_helpers() {
        let mut opts: HashMap<String, HayashiValue> = HashMap::new();
        opts.insert("cap".into(), HayashiValue::Str("My Table".into()));
        opts.insert("dec".into(), HayashiValue::Int(4));
        opts.insert("long".into(), HayashiValue::Bool(true));
        opts.insert("alpha".into(), HayashiValue::Float(0.05));

        assert_eq!(opt_str(&opts, "cap", "default"), "My Table");
        assert_eq!(opt_str(&opts, "missing", "default"), "default");
        assert_eq!(opt_int(&opts, "dec", 3), 4);
        assert_eq!(opt_int(&opts, "missing", 3), 3);
        assert_eq!(opt_bool(&opts, "long", false), true);
        assert_eq!(opt_bool(&opts, "missing", false), false);
        assert!((opt_f64(&opts, "alpha", 1.0) - 0.05).abs() < 1e-10);
        assert!((opt_f64(&opts, "missing", 1.0) - 1.0).abs() < 1e-10);
    }
}
