mod contributions;
mod languages;
mod statistics;

pub use contributions::Contributions;
pub use languages::Languages;
pub use statistics::Statistics;

use crate::theme::Theme;

/// Configuration for rendering a tile
pub struct RenderConfig<'a> {
    pub username: &'a str,
    pub show_username: bool,
    pub theme: Theme,
}

impl<'a> RenderConfig<'a> {
    pub fn new(username: &'a str, show_username: bool, theme: Theme) -> Self {
        Self {
            username,
            show_username,
            theme,
        }
    }

    /// Generate title with optional username prefix
    pub fn title(&self, base_title: &str) -> String {
        if self.show_username {
            format!("{}'s {}", self.username, base_title)
        } else {
            base_title.to_string()
        }
    }
}

/// Trait for generating SVG tiles
pub trait Tile {
    /// The base name of the tile (e.g., "statistics", "languages")
    fn name(&self) -> &'static str;

    /// Render the tile as an SVG string
    fn render(&self, config: &RenderConfig) -> String;

    /// Generate filename for the given theme
    fn filename(&self, theme_name: &str) -> String {
        format!("{}_{}.svg", self.name(), theme_name)
    }
}

/// Generate an empty placeholder SVG
pub fn empty_svg(message: &str, theme: Theme) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="350" height="100" viewBox="0 0 350 100">
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
  </style>
  <rect width="350" height="100" rx="4.5" fill="{}"/>
  <text x="175" y="55" fill="{}" font-size="14" text-anchor="middle">{}</text>
</svg>"#,
        theme.bg, theme.text, message
    )
}

/// Common CSS styles for SVG tiles
pub const SVG_STYLES: &str = "text { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }";
