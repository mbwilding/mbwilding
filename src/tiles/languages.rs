use std::collections::HashMap;

use super::{RenderConfig, SVG_STYLES, Tile, empty_svg};
use crate::github::User;

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
        let title = config.title("Languages");

        let top_langs: Vec<_> = self.languages.iter().take(8).collect();

        if top_langs.is_empty() {
            return empty_svg("No Languages Found", theme);
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
        let cx = 85.0;
        let cy = 85.0;
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
        for (i, (name, pct, color)) in lang_data.iter().enumerate() {
            let y = 55 + i * 22;
            legend.push_str(&format!(
                r#"<g transform="translate(25, {})">
                <rect width="12" height="12" rx="2" fill="{}"/>
                <text x="18" y="10" fill="{}" font-size="11">{} ({:.1}%)</text>
            </g>"#,
                y, color, theme.text, name, pct
            ));
        }

        // Dynamic height based on number of languages
        let legend_height = 55 + lang_data.len() * 22 + 15;
        let donut_height = 50 + 190 + 15;
        let height = legend_height.max(donut_height);

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="350" height="{}" viewBox="0 0 350 {}">
  <style>{}</style>
  <rect width="350" height="{}" rx="4.5" fill="{}"/>
  <text x="25" y="35" fill="{}" font-size="16" font-weight="600">{}</text>
  {}
  <g transform="translate(160, 50)">
    {}
  </g>
</svg>"#,
            height, height, SVG_STYLES, height, theme.bg, theme.title, title, legend, paths
        )
    }
}
