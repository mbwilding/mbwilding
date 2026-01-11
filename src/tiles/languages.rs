use super::{RenderConfig, SVG_STYLES, Tile, empty_svg};
use crate::github::User;
use std::collections::HashMap;

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
        let mut lang_bytes: HashMap<String, (u64, String)> = HashMap::new();

        for repo in &user.repositories.nodes {
            if repo.is_fork {
                continue;
            }
            if let Some(languages) = &repo.languages {
                for edge in &languages.edges {
                    let entry = lang_bytes
                        .entry(edge.node.name.clone())
                        .or_insert((0, edge.node.color.clone().unwrap_or("#858585".to_string())));
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
        let theme = config.theme;

        let top_langs: Vec<_> = self.languages.iter().take(8).collect();

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
        let cx = 70.0;
        let cy = 70.0;
        let r = 70.0;
        let inner_r = 42.0;

        let mut paths = String::new();
        let mut start_angle = -90.0_f64;

        for (_, pct, color) in &lang_data {
            let sweep = pct * 3.6;
            let end_angle = start_angle + sweep;

            let start_rad = start_angle.to_radians();
            let end_rad = end_angle.to_radians();

            let x1 = cx + r * start_rad.cos();
            let y1 = cy + r * start_rad.sin();
            let x2 = cx + r * end_rad.cos();
            let y2 = cy + r * end_rad.sin();
            let x3 = cx + inner_r * end_rad.cos();
            let y3 = cy + inner_r * end_rad.sin();
            let x4 = cx + inner_r * start_rad.cos();
            let y4 = cy + inner_r * start_rad.sin();

            let large_arc = if sweep > 180.0 { 1 } else { 0 };

            paths.push_str(&format!(
                r#"<path d="M {:.2} {:.2} A {} {} 0 {} 1 {:.2} {:.2} L {:.2} {:.2} A {} {} 0 {} 0 {:.2} {:.2} Z" fill="{}"/>"#,
                x1, y1, r, r, large_arc, x2, y2, x3, y3, inner_r, inner_r, large_arc, x4, y4, color
            ));

            start_angle = end_angle;
        }

        // Generate legend
        let mut legend = String::new();
        let row_height = 22;
        for (i, (name, pct, color)) in lang_data.iter().enumerate() {
            let y = i * row_height;
            legend.push_str(&format!(
                r#"<g transform="translate(0, {})">
                <rect width="12" height="12" rx="2" fill="{}"/>
                <text x="18" y="10" fill="{}" font-size="11">{} ({:.1}%)</text>
            </g>"#,
                y, color, theme.text, name, pct
            ));
        }

        // Dynamic height based on number of languages
        let legend_height = lang_data.len() * row_height;
        let donut_diameter = 140;
        let height = legend_height.max(donut_diameter);
        let width = 140 + 10 + 140; // legend width + gap + donut

        let bg_rect = if config.opaque {
            format!(
                r#"<rect width="{}" height="{}" rx="4.5" fill="{}"/>"#,
                width, height, theme.bg
            )
        } else {
            String::new()
        };

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <style>{}</style>
  {}
  {}
  <g transform="translate(150, {})">
    {}
  </g>
</svg>"#,
            width,
            height,
            width,
            height,
            SVG_STYLES,
            bg_rect,
            legend,
            (height - donut_diameter) / 2,
            paths
        )
    }
}
