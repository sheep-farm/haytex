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

use hayashi_plugin_sdk::{hayashi_fn, hayashi_plugin};
use hayashi_plugin_sdk::value::{HayashiValue, FromHayashi, IntoHayashi};
use std::collections::HashMap;

hayashi_plugin!();

mod helpers;
mod models;
mod tables;
mod equations;
mod format;

// Re-export the main functions
pub use tables::{table, regression, summary, correlation, codebook, anova, tests, diagnostics};
pub use equations::{equation, matrix, margins, forecast};
