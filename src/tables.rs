//! Table-generating functions: table, regression, summary, correlation,
//! codebook, anova, tests, diagnostics.

use crate::helpers::*;
use crate::models::{ModelData, DfData};
use hayashi_plugin_sdk::{hayashi_fn, value::HayashiValue};
use std::collections::HashMap;

/// 1. haytex::table(df, opts)
/// DataFrame → tabular with booktabs.
/// opts: caption="", label="", decimals=3, longtable=false
#[hayashi_fn]
pub fn table(
    df: HayashiValue,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let data = match DfData::from_value(&df) {
        Ok(d) => d,
        Err(e) => return format!("% Error: {e}"),
    };
    let decimals = opt_int(&opts, "decimals", 3) as usize;
    let caption = opt_str(&opts, "caption", "");
    let label = opt_str(&opts, "label", "");
    let longtable = opt_bool(&opts, "longtable", false);
    let ncols = data.columns.len();
    let nrows = data.nrow();

    let env = if longtable { "longtable" } else { "tabular" };
    let align = format!("l{}", repeat("r", ncols.saturating_sub(1)));

    let mut s = String::new();
    s.push_str(&wrap_table_start(&caption, &label));
    s.push_str(&format!("\\begin{{{env}}}{{{align}}}\n\\toprule\n"));

    // Header
    let hdr: Vec<String> = data.columns.iter().map(|c| esc(c)).collect();
    s.push_str(&join(&hdr, " & "));
    s.push_str(" \\\\\n\\midrule\n");

    // Body
    for i in 0..nrows {
        let row: Vec<String> = data.columns.iter().map(|col| {
            if let Some(floats) = data.data.get(col) {
                fmt_trim(floats[i], decimals)
            } else if let Some(strings) = data.data_str.get(col) {
                esc(&strings[i])
            } else {
                "---".to_string()
            }
        }).collect();
        s.push_str(&join(&row, " & "));
        s.push_str(" \\\\\n");
    }

    s.push_str("\\bottomrule\n");
    s.push_str(&format!("\\end{{{env}}}\n"));
    s.push_str(&wrap_table_end(&caption));
    s
}

/// 2. haytex::regression(models, opts)
/// Multiple models side by side, publication-ready.
/// opts: title="", label="", labels={}, stars=true, se=true, decimals=3,
///       stats=["n", "r2"]
#[hayashi_fn]
pub fn regression(
    models: Vec<HayashiValue>,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let decimals = opt_int(&opts, "decimals", 3) as usize;
    let show_stars = opt_bool(&opts, "stars", true);
    let show_se = opt_bool(&opts, "se", true);
    let title = opt_str(&opts, "title", "");
    let label = opt_str(&opts, "label", "");
    let var_labels = opt_labels(&opts, "labels");

    // Default stats
    let stats_list = match opts.get("stats") {
        Some(HayashiValue::List(lst)) => lst.iter().map(|v| match v {
            HayashiValue::Str(s) => s.clone(),
            _ => format!("{:?}", v),
        }).collect(),
        _ => vec!["n".to_string(), "r2".to_string()],
    };

    // Extract model data
    let model_data: Vec<ModelData> = models.iter()
        .filter_map(|m| ModelData::from_value(m).ok())
        .collect();
    let nmodels = model_data.len();
    if nmodels == 0 {
        return "% Error: no valid models".to_string();
    }

    // Collect all variable names (union, preserving order)
    let mut all_vars: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for md in &model_data {
        for v in &md.variables {
            if !seen.contains(v) {
                all_vars.push(v.clone());
                seen.insert(v.clone());
            }
        }
    }

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str(&format!("\\begin{{tabular}}{{l{}}}\n\\toprule\n", repeat("c", nmodels)));

    // Model headers: (1), (2), ...
    let model_names: Vec<String> = (0..nmodels).map(|i| format!("({})", i + 1)).collect();
    s.push_str(" & ");
    s.push_str(&join(&model_names, " & "));
    s.push_str(" \\\\\n\\midrule\n");

    // Coefficient rows
    for vname in &all_vars {
        let display_name = if vname == "_cons" || vname == "const" {
            "Constant".to_string()
        } else if let Some(lbl) = var_labels.get(vname) {
            esc(lbl)
        } else {
            esc(vname)
        };

        let mut coef_row: Vec<String> = Vec::new();
        let mut se_row: Vec<String> = Vec::new();

        for md in &model_data {
            if let Some(idx) = md.find_var(vname) {
                let c = md.coef[idx];
                let se = md.std_err[idx];
                let p = md.p_value[idx];
                let star = if show_stars { stars(p) } else { "" };
                coef_row.push(format!("{}{}", fmt_trim(c, decimals), star));
                if show_se {
                    se_row.push(format!("({})", fmt_trim(se, decimals)));
                }
            } else {
                coef_row.push(String::new());
                if show_se {
                    se_row.push(String::new());
                }
            }
        }

        s.push_str(&display_name);
        s.push_str(" & ");
        s.push_str(&join(&coef_row, " & "));
        s.push_str(" \\\\\n");
        if show_se {
            s.push_str(" & ");
            s.push_str(&join(&se_row, " & "));
            s.push_str(" \\\\\n");
        }
    }

    s.push_str("\\midrule\n");

    // Summary statistics rows
    for stat_name in &stats_list {
        let mut row: Vec<String> = Vec::new();
        for md in &model_data {
            let val = if stat_name == "n" {
                md.get_stat("n").map(|v| format!("{}", v as i64)).unwrap_or_default()
            } else {
                md.get_stat(stat_name).map(|v| fmt_trim(v, decimals)).unwrap_or_default()
            };
            row.push(val);
        }
        let stat_label = stat_label(stat_name);
        s.push_str(&stat_label);
        s.push_str(" & ");
        s.push_str(&join(&row, " & "));
        s.push_str(" \\\\\n");
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    if show_stars {
        s.push_str(star_legend());
    }
    s
}

/// 3. haytex::summary(df, vars, opts)
/// Descriptive statistics table.
/// opts: decimals=3, title="", label=""
#[hayashi_fn]
pub fn summary(
    df: HayashiValue,
    vars: Vec<String>,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let data = match DfData::from_value(&df) {
        Ok(d) => d,
        Err(e) => return format!("% Error: {e}"),
    };
    let decimals = opt_int(&opts, "decimals", 3) as usize;
    let title = opt_str(&opts, "title", "");
    let label = opt_str(&opts, "label", "");

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{lrrrrrrr}\n\\toprule\n");
    s.push_str("Variable & N & Mean & Std.~Dev. & Min & p25 & p50 & Max \\\\\n\\midrule\n");

    for vname in &vars {
        let col = match data.get_col(vname) {
            Ok(c) => c,
            Err(e) => {
                s.push_str(&format!("% Error: {e}\n"));
                continue;
            }
        };
        let vals: Vec<f64> = col.iter().copied().filter(|x| !x.is_nan()).collect();
        let nv = vals.len();
        if nv == 0 {
            s.push_str(&format!("{} & 0 & --- & --- & --- & --- & --- & --- \\\\\n", esc(vname)));
            continue;
        }
        let mean = vals.iter().sum::<f64>() / nv as f64;
        let variance = if nv > 1 {
            vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nv - 1) as f64
        } else { 0.0 };
        let sd = variance.sqrt();
        let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Quantiles via sorting
        let mut sorted = vals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let q25 = sorted[nv / 4];
        let q50 = sorted[nv / 2];

        s.push_str(&format!(
            "{} & {} & {} & {} & {} & {} & {} & {} \\\\\n",
            esc(vname), nv,
            fmt_trim(mean, decimals), fmt_trim(sd, decimals),
            fmt_trim(mn, decimals), fmt_trim(q25, decimals),
            fmt_trim(q50, decimals), fmt_trim(mx, decimals)
        ));
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s
}

/// 6. haytex::correlation(df, vars, opts)
/// Correlation matrix with stars.
/// opts: decimals=3, title="", label=""
#[hayashi_fn]
pub fn correlation(
    df: HayashiValue,
    vars: Vec<String>,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let data = match DfData::from_value(&df) {
        Ok(d) => d,
        Err(e) => return format!("% Error: {e}"),
    };
    let decimals = opt_int(&opts, "decimals", 3) as usize;
    let title = opt_str(&opts, "title", "");
    let label = opt_str(&opts, "label", "");
    let nvars = vars.len();

    // Extract columns
    let cols: Vec<Vec<f64>> = vars.iter()
        .filter_map(|v| data.get_col(v).ok().cloned())
        .collect();
    if cols.len() != nvars {
        return format!("% Error: could not extract all {} columns", nvars);
    }

    // Compute means
    let means: Vec<f64> = cols.iter().map(|c| {
        c.iter().sum::<f64>() / c.len() as f64
    }).collect();

    // Compute correlation matrix
    let n = cols[0].len();
    let mut cor = vec![vec![0.0f64; nvars]; nvars];
    for i in 0..nvars {
        for j in 0..nvars {
            if i == j {
                cor[i][j] = 1.0;
            } else if j > i {
                let mut cov = 0.0;
                let mut vi2 = 0.0;
                let mut vj2 = 0.0;
                for k in 0..n {
                    let di = cols[i][k] - means[i];
                    let dj = cols[j][k] - means[j];
                    cov += di * dj;
                    vi2 += di * di;
                    vj2 += dj * dj;
                }
                let r = cov / (vi2 * vj2).sqrt();
                cor[i][j] = r;
                cor[j][i] = r;
            }
        }
    }

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str(&format!("\\begin{{tabular}}{{l{}}}\n\\toprule\n", repeat("c", nvars)));

    // Header
    let hdr: Vec<String> = std::iter::once(String::new())
        .chain((0..nvars).map(|j| format!("({})", j + 1)))
        .collect();
    s.push_str(&join(&hdr, " & "));
    s.push_str(" \\\\\n\\midrule\n");

    // Lower triangle with stars
    for i in 0..nvars {
        let mut row: Vec<String> = vec![format!("({}) {}", i + 1, esc(&vars[i]))];
        for j in 0..nvars {
            if j > i {
                row.push(String::new());
            } else {
                let r = cor[i][j];
                // Approximate p-value via t-statistic
                let star = if i != j && n > 2 {
                    let tstat = r.abs() * ((n - 2) as f64 / (1.0 - r * r)).sqrt();
                    if tstat > 2.576 { "***" }
                    else if tstat > 1.96 { "**" }
                    else if tstat > 1.645 { "*" }
                    else { "" }
                } else { "" };
                row.push(format!("{}{}", fmt_trim(r, decimals), star));
            }
        }
        s.push_str(&join(&row, " & "));
        s.push_str(" \\\\\n");
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s.push_str(star_legend());
    s
}

/// 9. haytex::codebook(df, opts)
/// Variable description table for appendix.
/// opts: title="", label=""
#[hayashi_fn]
pub fn codebook(
    df: HayashiValue,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let data = match DfData::from_value(&df) {
        Ok(d) => d,
        Err(e) => return format!("% Error: {e}"),
    };
    let title = opt_str(&opts, "title", "Variable Codebook");
    let label = opt_str(&opts, "label", "");

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{llrrr}\n\\toprule\n");
    s.push_str("Variable & Type & N & Unique & Missing \\\\\n\\midrule\n");

    for col in &data.columns {
        let n = data.nrow();
        let (n_unique, missing, col_type) = if let Some(floats) = data.data.get(col) {
            let missing = floats.iter().filter(|x| x.is_nan()).count();
            let unique: std::collections::HashSet<u64> = floats.iter()
                .filter(|x| !x.is_nan())
                .map(|x| x.to_bits())
                .collect();
            (unique.len(), missing, "numeric")
        } else if let Some(strings) = data.data_str.get(col) {
            let missing = strings.iter().filter(|s| s.is_empty()).count();
            let unique: std::collections::HashSet<&String> = strings.iter().filter(|s| !s.is_empty()).collect();
            (unique.len(), missing, "string")
        } else {
            (0, 0, "unknown")
        };

        s.push_str(&format!(
            "{} & {} & {} & {} & {} \\\\\n",
            esc(col), col_type, n, n_unique, missing
        ));
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s
}

/// 12. haytex::anova(model, opts)
/// ANOVA table: SS, df, MS, F, p.
/// opts: decimals=4, title="", label=""
#[hayashi_fn]
pub fn anova(
    model: HayashiValue,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let md = match ModelData::from_value(&model) {
        Ok(m) => m,
        Err(e) => return format!("% Error: {e}"),
    };
    let decimals = opt_int(&opts, "decimals", 4) as usize;
    let title = opt_str(&opts, "title", "ANOVA");
    let label = opt_str(&opts, "label", "");

    let n = md.n_obs().unwrap_or(0.0) as i64;
    let k = md.n_coef() as i64;
    let f_stat = md.get_stat("f_stat").unwrap_or(f64::NAN);
    let prob_f = md.get_stat("prob_f").unwrap_or(f64::NAN);
    let r2 = md.r_squared().unwrap_or(f64::NAN);
    let sigma = md.get_stat("sigma").unwrap_or(f64::NAN);

    let df_model = k - 1;
    let df_resid = n - k;
    let ss_resid = sigma * sigma * df_resid as f64;
    let ss_model = if r2.is_finite() && (1.0 - r2).abs() > 1e-12 {
        r2 * ss_resid * df_model as f64 / ((1.0 - r2) * df_resid as f64)
    } else { f64::NAN };
    let ss_total = ss_model + ss_resid;
    let ms_model = ss_model / df_model as f64;
    let ms_resid = ss_resid / df_resid as f64;

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{lrrrrr}\n\\toprule\n");
    s.push_str("Source & SS & df & MS & F & p \\\\\n\\midrule\n");
    s.push_str(&format!(
        "Model & {} & {} & {} & {} & {} \\\\\n",
        fmt_trim(ss_model, decimals), df_model,
        fmt_trim(ms_model, decimals),
        fmt_trim(f_stat, decimals), fmt_trim(prob_f, decimals)
    ));
    s.push_str(&format!(
        "Residual & {} & {} & {} & & \\\\\n",
        fmt_trim(ss_resid, decimals), df_resid, fmt_trim(ms_resid, decimals)
    ));
    s.push_str("\\midrule\n");
    s.push_str(&format!(
        "Total & {} & {} & & & \\\\\n",
        fmt_trim(ss_total, decimals), n - 1
    ));
    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s
}

/// 8. haytex::tests(tests, opts)
/// Generic hypothesis test table.
/// tests: list of dicts with name, stat, df, p
/// opts: decimals=4, title="", label=""
#[hayashi_fn]
pub fn tests(
    tests: Vec<HayashiValue>,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let decimals = opt_int(&opts, "decimals", 4) as usize;
    let title = opt_str(&opts, "title", "Specification Tests");
    let label = opt_str(&opts, "label", "");

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{lrll}\n\\toprule\n");
    s.push_str("Test & Statistic & df & p-value \\\\\n\\midrule\n");

    for t in &tests {
        if let HayashiValue::Dict(d) = t {
            let name = match d.get("name") {
                Some(HayashiValue::Str(s)) => esc(s),
                _ => "---".to_string(),
            };
            let stat = val_as_f64(d.get("stat").unwrap_or(&HayashiValue::Nil)).unwrap_or(f64::NAN);
            let df_val = match d.get("df") {
                Some(HayashiValue::Int(i)) => i.to_string(),
                Some(HayashiValue::Float(f)) => format!("{}", *f as i64),
                _ => String::new(),
            };
            let p = val_as_f64(d.get("p").unwrap_or(&HayashiValue::Nil)).unwrap_or(f64::NAN);
            let star = stars(p);
            s.push_str(&format!(
                "{} & {} & {} & {}{} \\\\\n",
                name, fmt_trim(stat, decimals), df_val, fmt_trim(p, decimals), star
            ));
        }
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    s.push_str(star_legend());
    s
}

/// 7. haytex::diagnostics(model, opts)
/// Diagnostic tests table. Returns a template for VIF, DW, BP, RESET, White, JB.
/// The user fills in the values from Hayashi's diagnostic functions.
/// opts: decimals=4, title="", label=""
#[hayashi_fn]
pub fn diagnostics(
    _model: HayashiValue,
    opts: HashMap<String, HayashiValue>,
) -> String {
    let decimals = opt_int(&opts, "decimals", 4) as usize;
    let title = opt_str(&opts, "title", "Diagnostic Tests");
    let label = opt_str(&opts, "label", "");

    let mut s = String::new();
    s.push_str(&wrap_table_start(&title, &label));
    s.push_str("\\begin{tabular}{lrrl}\n\\toprule\n");
    s.push_str("Test & Statistic & p-value & Result \\\\\n\\midrule\n");

    // Standard diagnostic tests with placeholder values
    let tests = [
        ("VIF (max)", "---", "---", "Check"),
        ("Durbin-Watson", "---", "---", "Check"),
        ("Breusch-Pagan", "---", "---", "Check"),
        ("RESET", "---", "---", "Check"),
        ("White", "---", "---", "Check"),
        ("Jarque-Bera", "---", "---", "Check"),
    ];

    for (name, stat, p, result) in &tests {
        s.push_str(&format!("{} & {} & {} & {} \\\\\n", name, stat, p, result));
    }

    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s.push_str(&wrap_table_end(&title));
    let _ = decimals; // reserved for future use when tests are computed
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

fn stat_label(name: &str) -> String {
    match name {
        "n" => "Observations".to_string(),
        "r2" => "$R^2$".to_string(),
        "adj_r2" => "Adj. $R^2$".to_string(),
        "pseudo_r2" => "Pseudo $R^2$".to_string(),
        "aic" => "AIC".to_string(),
        "bic" => "BIC".to_string(),
        "log_lik" => "Log Likelihood".to_string(),
        "f_stat" => "F-statistic".to_string(),
        "sigma" => "$\\sigma$".to_string(),
        "j_stat" => "Hansen J".to_string(),
        "j_p_value" => "J p-value".to_string(),
        "df_overid" => "Over-id. df".to_string(),
        "sigma_u" => "$\\sigma_u$".to_string(),
        "sigma_e" => "$\\sigma_e$".to_string(),
        "theta" => "$\\theta$".to_string(),
        "tau" => "$\\tau$".to_string(),
        "alpha" => "$\\alpha$".to_string(),
        "rho" => "$\\rho$".to_string(),
        "delta" => "$\\delta$".to_string(),
        "deviance" => "Deviance".to_string(),
        "qic" => "QIC".to_string(),
        "n_entities" => "Entities".to_string(),
        "n_groups" => "Groups".to_string(),
        "n_censored" => "Censored".to_string(),
        "sigma2" => "$\\sigma^2$".to_string(),
        _ => name.to_string(),
    }
}
