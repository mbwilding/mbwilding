use super::{BORDER_RADIUS, FONT_SIZE_SMALL, RenderConfig, SVG_STYLES, Tile, empty_svg};
use crate::github::User;
use log::debug;
use std::collections::HashMap;

// Layout constants
const DEFAULT_LANGUAGE_COLOR: &str = "#858585";
const MAX_LANGUAGES: usize = 8;

// Donut chart constants
const DONUT_CX: f64 = 70.0;
const DONUT_CY: f64 = 70.0;
const DONUT_OUTER_RADIUS: f64 = 70.0;
const DONUT_INNER_RADIUS: f64 = 42.0;
const START_ANGLE_DEGREES: f64 = -90.0;
const DEGREES_PER_PERCENT: f64 = 3.6; // 360.0 / 100.0
const LARGE_ARC_THRESHOLD: f64 = 180.0;

// Legend constants
const LEGEND_ROW_HEIGHT: usize = 16;
const LEGEND_ITEM_HEIGHT: usize = 12;
const LEGEND_RECT_SIZE: usize = 12;
const LEGEND_RECT_RADIUS: usize = 2;
const LEGEND_TEXT_X: usize = 18;
const LEGEND_TEXT_Y: usize = 10;
const LEGEND_WIDTH: usize = 140;
const LEGEND_DONUT_GAP: usize = 10;

/// Language with byte count and color
pub struct LanguageEntry {
    pub name: String,
    pub bytes: u64,
    pub color: String,
}

/// Languages data extracted from GitHub user
pub struct Languages {
    pub languages: Vec<LanguageEntry>,
    pub total_bytes: u64,
}

impl Languages {
    /// Extract language statistics from GitHub user data
    pub fn from_user(user: &User) -> Self {
        debug!("Extracting language statistics");
        let mut lang_bytes: HashMap<String, (u64, String)> = HashMap::new();

        for repo in &user.repositories.nodes {
            if repo.is_fork {
                continue;
            }
            if let Some(languages) = &repo.languages {
                for edge in &languages.edges {
                    let entry = lang_bytes.entry(edge.node.name.clone()).or_insert((
                        0,
                        edge.node
                            .color
                            .clone()
                            .unwrap_or(DEFAULT_LANGUAGE_COLOR.to_string()),
                    ));
                    entry.0 += edge.size;
                }
            }
        }

        let mut languages: Vec<_> = lang_bytes
            .into_iter()
            .map(|(name, (bytes, color))| LanguageEntry { name, bytes, color })
            .collect();

        languages.sort_by(|a, b| b.bytes.cmp(&a.bytes));

        let total_bytes: u64 = languages.iter().map(|l| l.bytes).sum();

        debug!(
            "Found {} languages, {} total bytes",
            languages.len(),
            total_bytes
        );
        for lang in languages.iter().take(MAX_LANGUAGES) {
            let pct = (lang.bytes as f64 / total_bytes as f64) * 100.0;
            debug!("{}: {} bytes ({:.1}%)", lang.name, lang.bytes, pct);
        }

        Self {
            languages,
            total_bytes,
        }
    }
}

impl Tile for Languages {
    fn name(&self) -> &'static str {
        "languages"
    }

    fn render(&self, config: &RenderConfig) -> String {
        debug!("Rendering languages tile");
        let theme = config.theme;

        let top_langs: Vec<_> = self.languages.iter().take(MAX_LANGUAGES).collect();

        if top_langs.is_empty() {
            return empty_svg("No Languages Found", theme, config.opaque);
        }

        // Calculate percentages
        let lang_data: Vec<_> = top_langs
            .iter()
            .map(|lang| {
                let pct = (lang.bytes as f64 / self.total_bytes as f64) * 100.0;
                (lang.name.as_str(), pct, lang.color.as_str())
            })
            .collect();

        // Generate donut chart
        let mut paths = String::new();
        let mut start_angle = START_ANGLE_DEGREES;

        for (_, pct, color) in &lang_data {
            let sweep = pct * DEGREES_PER_PERCENT;
            let end_angle = start_angle + sweep;

            let start_rad = start_angle.to_radians();
            let end_rad = end_angle.to_radians();

            let x1 = DONUT_CX + DONUT_OUTER_RADIUS * start_rad.cos();
            let y1 = DONUT_CY + DONUT_OUTER_RADIUS * start_rad.sin();
            let x2 = DONUT_CX + DONUT_OUTER_RADIUS * end_rad.cos();
            let y2 = DONUT_CY + DONUT_OUTER_RADIUS * end_rad.sin();
            let x3 = DONUT_CX + DONUT_INNER_RADIUS * end_rad.cos();
            let y3 = DONUT_CY + DONUT_INNER_RADIUS * end_rad.sin();
            let x4 = DONUT_CX + DONUT_INNER_RADIUS * start_rad.cos();
            let y4 = DONUT_CY + DONUT_INNER_RADIUS * start_rad.sin();

            let large_arc = if sweep > LARGE_ARC_THRESHOLD { 1 } else { 0 };

            paths.push_str(&format!(
                r#"<path d="M {:.2} {:.2} A {} {} 0 {} 1 {:.2} {:.2} L {:.2} {:.2} A {} {} 0 {} 0 {:.2} {:.2} Z" fill="{}"/>"#,
                x1, y1, DONUT_OUTER_RADIUS, DONUT_OUTER_RADIUS, large_arc, x2, y2, x3, y3, DONUT_INNER_RADIUS, DONUT_INNER_RADIUS, large_arc, x4, y4, color
            ));

            start_angle = end_angle;
        }

        // Generate legend
        let mut legend = String::new();
        for (i, (name, pct, color)) in lang_data.iter().enumerate() {
            let y = i * LEGEND_ROW_HEIGHT;
            legend.push_str(&format!(
                r#"<g transform="translate(0, {})">
                <rect width="{}" height="{}" rx="{}" fill="{}"/>
                <text x="{}" y="{}" fill="{}" font-size="{}">{} <tspan fill="{}">{:.1}%</tspan></text>
            </g>"#,
                y,
                LEGEND_RECT_SIZE,
                LEGEND_RECT_SIZE,
                LEGEND_RECT_RADIUS,
                color,
                LEGEND_TEXT_X,
                LEGEND_TEXT_Y,
                theme.text,
                FONT_SIZE_SMALL,
                name,
                theme.icon,
                pct
            ));
        }

        // Dynamic height based on number of languages
        let legend_height = (lang_data.len() - 1) * LEGEND_ROW_HEIGHT + LEGEND_ITEM_HEIGHT;
        let donut_diameter = (DONUT_OUTER_RADIUS * 2.0) as usize;
        let height = legend_height.max(donut_diameter);
        let width = LEGEND_WIDTH + LEGEND_DONUT_GAP + donut_diameter;

        // Center legend vertically if shorter than donut
        let legend_y_offset = (height as isize - legend_height as isize) / 2;
        let legend_translated = if legend_y_offset > 0 {
            format!(
                r#"<g transform="translate(0, {})">{}</g>"#,
                legend_y_offset, legend
            )
        } else {
            legend
        };

        // Center donut vertically
        let donut_y_offset = (height as isize - donut_diameter as isize) / 2;

        let bg_rect = if config.opaque {
            format!(
                r#"<rect width="{}" height="{}" rx="{}" fill="{}"/>"#,
                width, height, BORDER_RADIUS, theme.bg
            )
        } else {
            String::new()
        };

        let donut_x_offset = LEGEND_WIDTH + LEGEND_DONUT_GAP;
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <style>{}</style>
  {}
  {}
  <g transform="translate({}, {})">
    {}
  </g>
</svg>"#,
            width,
            height,
            width,
            height,
            SVG_STYLES,
            bg_rect,
            legend_translated,
            donut_x_offset,
            donut_y_offset,
            paths
        )
    }
}
