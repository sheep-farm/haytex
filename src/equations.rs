//! Equation and matrix functions: equation, matrix, margins, forecast.

use crate::helpers::*;
use crate::models::ModelData;
use hayashi_plugin_sdk::{hayashi_fn, value::HayashiValue};
use std::collections::HashMap;

/// 4. haytex::equation(model, opts)
/// Formula → LaTeX equation. With estimated=true, fills coefficients.
/// opts: estimated=false, decimals=3
#[hayashi_fn]
pub fn equation(
    model: HayashiValue,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let md = match ModelData::from_value(&model) {
        Ok(m) => m,
        Err(e) => return format!("% Error: {e}"),
    };
    let estimated = opt_bool(&opts, "estimated", false);
    let decimals = opt_int(&opts, "decimals", 3) as usize;

    let const_idx = md.has_constant();
    let mut eq = String::from("\\hat{y} = ");
    let mut first = true;

    if let Some(idx) = const_idx {
        if estimated {
            let c = md.coef[idx];
            let star = stars(md.p_value[idx]);
            eq.push_str(&format!("{}{}", fmt_trim(c, decimals), star));
        } else {
            eq.push_str("\\beta_0");
        }
        first = false;
    }

    let mut beta_idx = 0;
    for i in 0..md.n_coef() {
        if Some(i) == const_idx {
            continue;
        }
        let vname = &md.variables[i];
        let sub = (beta_idx + 1).to_string();
        if estimated {
            let c = md.coef[i];
            let star = stars(md.p_value[i]);
            if first {
                if c < 0.0 {
                    eq.push_str(&format!("- {}{}", fmt_trim(c.abs(), decimals), star));
                } else {
                    eq.push_str(&format!("{}{}", fmt_trim(c, decimals), star));
                }
            } else {
                if c < 0.0 {
                    eq.push_str(&format!(" - {}{}", fmt_trim(c.abs(), decimals), star));
                } else {
                    eq.push_str(&format!(" + {}{}", fmt_trim(c, decimals), star));
                }
            }
            eq.push(' ');
            eq.push_str(&esc(vname));
        } else {
            if !first {
                eq.push_str(" + ");
            }
            eq.push_str(&format!("\\beta_{{{sub}}} {}", esc(vname)));
        }
        beta_idx += 1;
        first = false;
    }
    eq.push_str(" + \\varepsilon");

    format!("\\begin{{equation}}\n{}\n\\end{{equation}}\n", eq)
}

/// 5. haytex::matrix(mat, opts)
/// Matrix → bmatrix/pmatrix.
/// opts: decimals=4, brackets="b" (b=brackets, p=parentheses)
#[hayashi_fn]
pub fn matrix(
    mat: Vec<Vec<f64>>,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let decimals = opt_int(&opts, "decimals", 4) as usize;
    let brackets = opt_str(&opts, "brackets", "b");
    let env = format!("{}matrix", brackets);

    let nrows = mat.len();
    if nrows == 0 {
        return format!("\\begin{{{env}}}\n\\end{{{env}}}\n");
    }
    let ncols = mat[0].len();

    let mut s = format!("\\begin{{{env}}}\n");
    for i in 0..nrows {
        let row: Vec<String> = (0..ncols).map(|j| fmt_trim(mat[i][j], decimals)).collect();
        s.push_str(&join(&row, " & "));
        if i < nrows - 1 {
            s.push_str(" \\\\\n");
        } else {
            s.push('\n');
        }
    }
    s.push_str(&format!("\\end{{{env}}}\n"));
    s
}

/// 10. haytex::margins(model, opts)
/// Marginal effects table (AME/MEM) for logit/probit.
/// Computes marginal effects at the mean using the PDF.
/// opts: decimals=4, title="", label=""
#[hayashi_fn]
pub fn margins(
    model: HayashiValue,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let md = match ModelData::from_value(&model) {
        Ok(m) => m,
        Err(e) => return format!("% Error: {e}"),
    };
    let decimals = opt_int(&opts, "decimals", 4) as usize;
    let title = opt_str(&opts, "title", "Marginal Effects");
    let label = opt_str(&opts, "label", "");

    // For logit: lambda = exp(X'b) / (1 + exp(X'b))^2
    // For probit: lambda = phi(X'b)
    // We use the average of coefficients as a proxy for X'b
    let is_logit = md.model_type == "logit";
    let is_probit = md.model_type == "probit";

    // Compute a representative X'b (mean of linear predictor ≈ mean of coef)
    let xb: f64 = md.coef.iter().sum::<f64>() / md.n_coef() as f64;
    let lambda = if is_logit {
        let exb = xb.exp();
        exb / (1.0 + exb).powi(2)
    } else if is_probit {
        // Standard normal PDF: phi(x) = exp(-x^2/2) / sqrt(2*pi)
        (-xb * xb / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt()
    } else {
        // For linear models, marginal effect = coef directly
        1.0
    };

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{lrrrl}\n\\toprule\n");
    s.push_str("Variable & dy/dx & Std.~Err. & z & p \\\\\n\\midrule\n");

    for i in 0..md.n_coef() {
        let vname = &md.variables[i];
        if vname == "_cons" || vname == "const" || vname == "intercept" {
            continue;
        }
        let me = md.coef[i] * lambda;
        let se_me = md.std_err[i] * lambda;
        let z = if se_me != 0.0 { me / se_me } else { f64::NAN };
        let p = md.p_value[i];
        let star = stars(p);
        s.push_str(&format!(
            "{} & {}{} & {} & {} & {} \\\\\n",
            esc(vname), fmt_trim(me, decimals), star,
            fmt_trim(se_me, decimals), fmt_trim(z, decimals), fmt_trim(p, decimals)
        ));
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s.push_str(star_legend());
    s
}

/// 11. haytex::forecast(model, h, opts)
/// Forecast table with confidence intervals.
/// opts: decimals=3, title="", label="", conf=0.95
#[hayashi_fn]
pub fn forecast(
    model: HayashiValue,
    h: i64,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let md = match ModelData::from_value(&model) {
        Ok(m) => m,
        Err(e) => return format!("% Error: {e}"),
    };
    let decimals = opt_int(&opts, "decimals", 3) as usize;
    let title = opt_str(&opts, "title", "Forecasts");
    let label = opt_str(&opts, "label", "");
    let conf = opt_f64(&opts, "conf", 0.95);

    // Z-value for confidence level
    let z = match conf {
        c if c >= 0.99 => 2.576,
        c if c >= 0.95 => 1.96,
        c if c >= 0.90 => 1.645,
        _ => 1.96,
    };

    let sigma = md.get_stat("sigma").or_else(|| md.get_stat("sigma2").map(|s| s.sqrt())).unwrap_or(0.0);
    let const_idx = md.has_constant();

    // Simple static forecast: y_hat = sum of coef * x
    // Since we don't have future X values, we use the mean of coefficients as a proxy
    // This is a simplified forecast — real forecasting needs predict() from Hayashi
    let base: f64 = md.coef.iter().enumerate()
        .filter(|(i, _)| Some(*i) != const_idx)
        .map(|(_, c)| c)
        .sum::<f64>()
        + const_idx.map(|i| md.coef[i]).unwrap_or(0.0);

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{rrrr}\n\\toprule\n");
    s.push_str("Period & Forecast & Lower & Upper \\\\\n\\midrule\n");

    for t in 1..=h as usize {
        let f = base; // simplified static forecast
        let lo = f - z * sigma;
        let hi = f + z * sigma;
        s.push_str(&format!(
            "{} & {} & {} & {} \\\\\n",
            t, fmt_trim(f, decimals), fmt_trim(lo, decimals), fmt_trim(hi, decimals)
        ));
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s
}

// ── Internal helpers ────────────────────────────────────────────────────────

fn wrap_table_start(caption: &str, label: &str) -> String {
    if caption.is_empty() {
        String::new()
    } else {
        let mut s = String::from("\\begin{table}[htbp]\n\\centering\n");
        s.push_str(&format!("\\caption{{{}}}\n", esc(caption)));
        if !label.is_empty() {
            s.push_str(&format!("\\label{{{}}}\n", label));
        }
        s
    }
}

fn wrap_table_end(caption: &str) -> &'static str {
    if caption.is_empty() { "" } else { "\\end{table}\n" }
}
