//! Output format renderers: LaTeX, HTML, RTF, CSV.
//!
//! Each renderer takes a generic table structure (header rows + body rows
//! + optional footer) and produces a string in the target format.

use crate::helpers::{esc, join, stars, fmt_trim};

/// Output format enum.
#[derive(Clone, Copy, PartialEq)]
pub enum Format {
    Latex,
    Html,
    Rtf,
    Csv,
}

impl Format {
    pub fn from_str(s: &str) -> Format {
        match s.to_lowercase().as_str() {
            "html" => Format::Html,
            "rtf" => Format::Rtf,
            "csv" => Format::Csv,
            _ => Format::Latex,
        }
    }

    /// Escape a cell value for this format.
    pub fn esc_cell(&self, s: &str) -> String {
        match self {
            Format::Latex => esc(s),
            Format::Html => s
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;"),
            Format::Rtf => s.replace('\\', "\\\\").replace('{', "\\{").replace('}', "\\}"),
            Format::Csv => s.replace('"', "\"\""),
        }
    }

    /// Escape a cell for HTML but preserve <sup> tags from star formatting.
    pub fn esc_cell_html(&self, s: &str) -> String {
        let mut result = String::new();
        let mut rest = s;
        loop {
            if let Some(start) = rest.find("<sup>") {
                result.push_str(&rest[..start]
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"));
                if let Some(end) = rest[start..].find("</sup>") {
                    // </sup> is 6 chars
                    let sup_end = start + end + 6;
                    result.push_str(&rest[start..sup_end]);
                    rest = &rest[sup_end..];
                } else {
                    result.push_str(&rest[start..]
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;"));
                    break;
                }
            } else {
                result.push_str(&rest
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"));
                break;
            }
        }
        result
    }

    /// Format a star string for this format (LaTeX uses raw, HTML uses <sup>).
    pub fn fmt_stars(&self, p: f64, show: bool) -> String {
        if !show {
            return String::new();
        }
        let s = stars(p);
        match self {
            Format::Latex | Format::Rtf => s.to_string(),
            Format::Html => if s.is_empty() { String::new() } else { format!("<sup>{}</sup>", s) },
            Format::Csv => s.to_string(),
        }
    }

    /// Star legend footer for this format.
    pub fn star_legend(&self) -> String {
        match self {
            Format::Latex => "\\par\\vspace{2pt}\n\\footnotesize \\*** p<0.01, ** p<0.05, * p<0.1\n".to_string(),
            Format::Html => "<p style=\"font-size:0.8em\">*** p&lt;0.01, ** p&lt;0.05, * p&lt;0.1</p>\n".to_string(),
            Format::Rtf => "\\par *** p<0.01, ** p<0.05, * p<0.1\\par\n".to_string(),
            Format::Csv => "*** p<0.01, ** p<0.05, * p<0.1\n".to_string(),
        }
    }
}

/// Generic table structure that all renderers consume.
pub struct Table {
    /// Column alignment: 'l', 'c', 'r' for each column.
    pub align: Vec<char>,
    /// Header rows (each is a Vec of cells).
    pub headers: Vec<Vec<String>>,
    /// Body rows (each is a Vec of cells).
    pub body: Vec<Vec<String>>,
    /// Footer rows (e.g. fit statistics), separated from body by a midrule.
    pub footer: Vec<Vec<String>>,
    /// Optional caption/title.
    pub caption: String,
    /// Optional label (LaTeX only).
    pub label: String,
    /// Whether to show star legend after the table.
    pub show_stars: bool,
}

impl Table {
    pub fn new(ncols: usize, default_align: char) -> Self {
        Table {
            align: vec![default_align; ncols],
            headers: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
            caption: String::new(),
            label: String::new(),
            show_stars: false,
        }
    }

    /// Render the table in the given format.
    pub fn render(&self, fmt: Format) -> String {
        match fmt {
            Format::Latex => self.render_latex(fmt),
            Format::Html => self.render_html(fmt),
            Format::Rtf => self.render_rtf(fmt),
            Format::Csv => self.render_csv(fmt),
        }
    }

    fn render_latex(&self, fmt: Format) -> String {
        let mut s = String::new();
        if !self.caption.is_empty() {
            s.push_str("\\begin{table}[htbp]\n\\centering\n");
            s.push_str(&format!("\\caption{{{}}}\n", esc(&self.caption)));
            if !self.label.is_empty() {
                s.push_str(&format!("\\label{{{}}}\n", self.label));
            }
        }
        let align: String = self.align.iter().collect();
        s.push_str(&format!("\\begin{{tabular}}{{{align}}}\n\\toprule\n"));
        for row in &self.headers {
            s.push_str(&join(row, " & "));
            s.push_str(" \\\\\n");
        }
        if !self.headers.is_empty() {
            s.push_str("\\midrule\n");
        }
        for row in &self.body {
            s.push_str(&join(row, " & "));
            s.push_str(" \\\\\n");
        }
        if !self.footer.is_empty() {
            s.push_str("\\midrule\n");
            for row in &self.footer {
                s.push_str(&join(row, " & "));
                s.push_str(" \\\\\n");
            }
        }
        s.push_str("\\bottomrule\n\\end{tabular}\n");
        if !self.caption.is_empty() {
            s.push_str("\\end{table}\n");
        }
        if self.show_stars {
            s.push_str(&fmt.star_legend());
        }
        s
    }

    fn render_html(&self, fmt: Format) -> String {
        let mut s = String::new();
        s.push_str("<table>\n");
        if !self.caption.is_empty() {
            s.push_str(&format!("<caption>{}</caption>\n", fmt.esc_cell(&self.caption)));
        }
        if !self.headers.is_empty() {
            s.push_str("<thead>\n");
            for row in &self.headers {
                s.push_str("<tr>");
                for cell in row {
                    s.push_str(&format!("<th>{}</th>", fmt.esc_cell_html(cell)));
                }
                s.push_str("</tr>\n");
            }
            s.push_str("</thead>\n");
        }
        s.push_str("<tbody>\n");
        for row in &self.body {
            s.push_str("<tr>");
            for cell in row {
                s.push_str(&format!("<td>{}</td>", fmt.esc_cell_html(cell)));
            }
            s.push_str("</tr>\n");
        }
        if !self.footer.is_empty() {
            s.push_str("</tbody>\n<tbody>\n");
            for row in &self.footer {
                s.push_str("<tr>");
                for cell in row {
                    s.push_str(&format!("<td>{}</td>", fmt.esc_cell_html(cell)));
                }
                s.push_str("</tr>\n");
            }
        }
        s.push_str("</tbody>\n</table>\n");
        if self.show_stars {
            s.push_str(&fmt.star_legend());
        }
        s
    }

    fn render_rtf(&self, fmt: Format) -> String {
        let mut s = String::new();
        s.push_str("{\\rtf1\\ansi\n");
        if !self.caption.is_empty() {
            s.push_str(&format!("\\par\\b {}\\b0\\par\n", fmt.esc_cell(&self.caption)));
        }
        let ncols = self.align.len();
        let page_w = 9000;
        let col_w = page_w / ncols as i32;
        let row_def: String = (1..=ncols)
            .map(|i| format!("\\cellx {}", col_w * i as i32))
            .collect::<Vec<_>>()
            .join(" ");

        let render_row = |row: &[String]| -> String {
            let mut r = format!("\\trowd {} ", row_def);
            for cell in row {
                r.push_str(&format!("\\intbl {} \\cell", fmt.esc_cell(cell)));
            }
            r.push_str("\\row\n");
            r
        };

        for row in &self.headers {
            s.push_str(&render_row(row));
        }
        for row in &self.body {
            s.push_str(&render_row(row));
        }
        for row in &self.footer {
            s.push_str(&render_row(row));
        }
        s.push_str("}\n");
        if self.show_stars {
            s.push_str(&fmt.star_legend());
        }
        s
    }

    fn render_csv(&self, fmt: Format) -> String {
        let mut s = String::new();
        for row in &self.headers {
            let cells: Vec<String> = row.iter()
                .map(|c| format!("\"{}\"", fmt.esc_cell(c)))
                .collect();
            s.push_str(&cells.join(","));
            s.push('\n');
        }
        for row in &self.body {
            let cells: Vec<String> = row.iter()
                .map(|c| format!("\"{}\"", fmt.esc_cell(c)))
                .collect();
            s.push_str(&cells.join(","));
            s.push('\n');
        }
        for row in &self.footer {
            let cells: Vec<String> = row.iter()
                .map(|c| format!("\"{}\"", fmt.esc_cell(c)))
                .collect();
            s.push_str(&cells.join(","));
            s.push('\n');
        }
        if self.show_stars {
            s.push_str(&format!("\"{}\"\n", fmt.star_legend().trim()));
        }
        s
    }
}

/// Format a coefficient with optional stars for the given format.
pub fn fmt_coef(coef: f64, p: f64, decimals: usize, fmt: Format, show_stars: bool) -> String {
    let star = fmt.fmt_stars(p, show_stars);
    format!("{}{}", fmt_trim(coef, decimals), star)
}

/// Format a standard error in parentheses.
pub fn fmt_se(se: f64, decimals: usize) -> String {
    format!("({})", fmt_trim(se, decimals))
}
