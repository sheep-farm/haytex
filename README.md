# haytex

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org)

LaTeX snippet generator plugin for [Hayashi](https://github.com/sheep-farm/hayashi) — publication-ready tables and equations from econometric models, DataFrames, matrices, and test results.

Native Rust plugin. All functions return `String` containing LaTeX code (no `\documentclass`, no preamble — just snippets ready to paste into a `.tex` file).

## Install

```bash
hay install sheep-farm/haytex
```

Or manually:

```bash
git clone https://github.com/sheep-farm/haytex.git
cd haytex
cargo build --release
cp target/release/libhaytex.so ~/.hay/packages/sheep-farm/haytex.so
```

## Usage

```
import("sheep-farm/haytex", as=haytex)

load "https://www.stata-press.com/data/r9/auto.dta" as auto

let m1 = ols(price ~ mpg + weight, auto)
let m2 = ols(price ~ mpg + weight + length, auto)

// Regression table — multiple models side by side
let tex = haytex::regression([m1, m2], {"title": "Car Price Models", "decimals": 3})
print(tex)

// Estimated equation with real coefficients
let eq = haytex::equation(m1, {"estimated": true, "decimals": 3})
print(eq)

// Save to file
write(tex, "table.tex")
```

## Functions

### Tables

#### `haytex::table(df, opts)`

DataFrame → `tabular` with `booktabs`.

```
let tex = haytex::table(df, {"caption": "My Data", "label": "tab:mydata", "decimals": 3, "longtable": false})
```

| Option      | Type    | Default | Description                          |
|-------------|---------|---------|--------------------------------------|
| `caption`   | string  | `""`    | Table caption (wraps in `table` env) |
| `label`     | string  | `""`    | LaTeX label for `\ref`               |
| `decimals`  | int     | `3`     | Decimal places for numeric columns   |
| `longtable` | bool    | `false` | Use `longtable` instead of `tabular` |

#### `haytex::regression(models, opts)`

Multiple models side by side — publication-ready regression table with coefficients, standard errors in parentheses, significance stars, and fit statistics.

```
let tex = haytex::regression([m1, m2, m3], {
    "title": "Results",
    "labels": {"mpg": "MPG", "weight": "Weight (lbs)"},
    "stars": true,
    "se": true,
    "decimals": 3,
    "stats": ["n", "r2", "adj_r2", "aic"]
})
```

| Option     | Type   | Default       | Description                              |
|------------|--------|---------------|------------------------------------------|
| `title`    | string | `""`          | Table caption                            |
| `label`    | string | `""`          | LaTeX label                              |
| `labels`   | dict   | `{}`          | Variable name → display label mapping    |
| `stars`    | bool   | `true`        | Show significance stars                  |
| `se`       | bool   | `true`        | Show standard errors in parentheses      |
| `decimals` | int    | `3`           | Decimal places                           |
| `stats`    | list   | `["n", "r2"]` | Fit statistics to display in footer      |

Supported `stats` keys: `n`, `r2`, `adj_r2`, `pseudo_r2`, `aic`, `bic`, `log_lik`, `f_stat`, `sigma`, `j_stat`, `j_p_value`, `df_overid`, `sigma_u`, `sigma_e`, `theta`, `tau`, `alpha`, `rho`, `delta`, `deviance`, `qic`, `n_entities`, `n_groups`, `n_censored`, `sigma2`.

#### `haytex::summary(df, vars, opts)`

Descriptive statistics table: N, Mean, Std. Dev., Min, p25, p50, Max.

```
let tex = haytex::summary(auto, ["price", "mpg", "weight"], {"decimals": 2, "title": "Summary"})
```

#### `haytex::correlation(df, vars, opts)`

Lower-triangular correlation matrix with significance stars.

```
let tex = haytex::correlation(auto, ["price", "mpg", "weight", "length"], {"decimals": 3})
```

#### `haytex::codebook(df, opts)`

Variable description table for appendix: name, type, N, unique values, missing count.

```
let tex = haytex::codebook(auto, {"title": "Variable Codebook"})
```

#### `haytex::anova(model, opts)`

ANOVA table: Source, SS, df, MS, F, p-value. Computed from model fit statistics.

```
let tex = haytex::anova(m1, {"decimals": 4})
```

#### `haytex::tests(tests, opts)`

Generic hypothesis test table. Accepts a list of dicts with `name`, `stat`, `df`, `p`.

```
let tex = haytex::tests([
    {"name": "Hausman", "stat": 12.3, "df": 5, "p": 0.03},
    {"name": "Hansen J", "stat": 3.45, "df": 2, "p": 0.178}
], {"decimals": 3, "title": "Specification Tests"})
```

#### `haytex::diagnostics(model, opts)`

Diagnostic tests template (VIF, Durbin-Watson, Breusch-Pagan, RESET, White, Jarque-Bera). Returns a structured template for the user to fill with test results.

```
let tex = haytex::diagnostics(m1, {"decimals": 4})
```

### Equations

#### `haytex::equation(model, opts)`

Formula → LaTeX equation. With `estimated=true`, fills in real coefficients with significance stars.

```
// Theoretical: \hat{y} = \beta_0 + \beta_1 x_1 + \beta_2 x_2 + \varepsilon
let eq1 = haytex::equation(m1, {})

// Estimated: \hat{y} = 1946.069 - 49.512 mpg + 1.747*** weight + \varepsilon
let eq2 = haytex::equation(m1, {"estimated": true, "decimals": 3})
```

| Option      | Type | Default | Description                          |
|-------------|------|---------|--------------------------------------|
| `estimated` | bool | `false` | Use estimated coefficients with stars |
| `decimals`  | int  | `3`     | Decimal places                       |

#### `haytex::matrix(mat, opts)`

Numeric matrix → `bmatrix` or `pmatrix`.

```
let tex = haytex::matrix([[1.0, 2.0], [3.0, 4.0]], {"decimals": 4, "brackets": "b"})
```

| Option     | Type   | Default | Description                    |
|------------|--------|---------|--------------------------------|
| `decimals` | int    | `4`     | Decimal places                 |
| `brackets` | string | `"b"`   | `b` for `bmatrix`, `p` for `pmatrix` |

#### `haytex::margins(model, opts)`

Marginal effects table for logit/probit. Computes marginal effects at the mean using the PDF (logistic or normal).

```
let m = logit(died ~ age + drug, cancer)
let tex = haytex::margins(m, {"decimals": 4, "title": "Marginal Effects"})
```

#### `haytex::forecast(model, h, opts)`

Forecast table with confidence intervals for `h` periods ahead.

```
let tex = haytex::forecast(m, 5, {"decimals": 2, "conf": 0.95, "title": "5-Period Forecast"})
```

| Option     | Type   | Default | Description                    |
|------------|--------|---------|--------------------------------|
| `decimals` | int    | `3`     | Decimal places                 |
| `conf`     | float  | `0.95`  | Confidence level (0.90/0.95/0.99) |
| `title`    | string | `"Forecasts"` | Table caption           |

## Supported model types

haytex works with any model that Hayashi can estimate. The following model types are fully supported with coefficient extraction and fit statistics:

| Model          | Estimator(s)                              |
|----------------|-------------------------------------------|
| OLS            | `ols`, `reg`                              |
| IV / 2SLS      | `iv`, `ivreg`                             |
| Logit / Probit | `logit`, `probit`                         |
| Panel FE       | `xtreg`, `fe`                             |
| Panel RE       | `xtreg`, `re`                             |
| GMM            | `gmm`                                     |
| Poisson        | `poisson`                                 |
| NegBin         | `negbin`                                  |
| GLM            | `glm`                                     |
| Quantile       | `rq`, `quantile`                          |
| Tobit          | `tobit`                                   |
| Heckman        | `heckman`                                 |
| Ordered        | `ologit`, `oprobit`                       |
| Arellano-Bond  | `xtabond`                                 |
| Ridge/Lasso    | `ridge`, `lasso`, `elasticnet`            |
| RLM            | `rlm`                                     |
| Beta           | `betareg`                                 |
| GEE            | `gee`                                     |
| ARIMA          | `arima`                                   |
| GARCH          | `garch`                                   |

## LaTeX requirements

The generated snippets use the following packages — add them to your preamble:

```latex
\usepackage{booktabs}    % \toprule, \midrule, \bottomrule
\usepackage{amsmath}     % \begin{equation}, \begin{bmatrix}
\usepackage{longtable}   % if using longtable=true
```

## License

MIT
