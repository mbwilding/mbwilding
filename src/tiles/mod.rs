mod contributions;
mod languages;
mod statistics;

use crate::theme::Theme;
pub use contributions::Contributions;
pub use languages::Languages;
pub use statistics::Statistics;

/// Configuration for rendering a tile
pub struct RenderConfig {
    pub theme: Theme,
    pub opaque: bool,
}

impl RenderConfig {
    pub fn new(theme: Theme, opaque: bool) -> Self {
        Self { theme, opaque }
    }
}

/// Trait for generating SVG tiles
pub trait Tile {
    /// The base name of the tile
    fn name(&self) -> &'static str;

    /// Render the tile as an SVG string
    fn render(&self, config: &RenderConfig) -> String;

    /// Generate filename for the given theme
    fn filename(&self, theme_name: &str) -> String {
        format!("{}_{}.svg", self.name(), theme_name)
    }
}

/// Generate an empty placeholder SVG
pub fn empty_svg(message: &str, theme: Theme, opaque: bool) -> String {
    let bg_rect = if opaque {
        format!(
            r#"<rect width="350" height="100" rx="4.5" fill="{}"/>"#,
            theme.bg
        )
    } else {
        String::new()
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="350" height="100" viewBox="0 0 350 100">
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
  </style>
  {}
  <text x="175" y="55" fill="{}" font-size="14" text-anchor="middle">{}</text>
</svg>"#,
        bg_rect, theme.text, message
    )
}

/// Common CSS styles for SVG tiles
pub const SVG_STYLES: &str = "text { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }";
