mod contributions;
mod languages;
mod statistics;

use crate::theme::Theme;
pub use contributions::Contributions;
pub use languages::Languages;
pub use statistics::Statistics;

// Shared layout constants
pub const CHAR_WIDTH: f64 = 6.5;
pub const BORDER_RADIUS: f64 = 4.5;
pub const FONT_SIZE: usize = 12;
pub const FONT_SIZE_SMALL: usize = 11;

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

// Empty SVG constants
const EMPTY_SVG_WIDTH: usize = 350;
const EMPTY_SVG_HEIGHT: usize = 100;
const EMPTY_SVG_FONT_SIZE: usize = 14;

/// Generate an empty placeholder SVG
pub fn empty_svg(message: &str, theme: Theme, opaque: bool) -> String {
    let text_x = EMPTY_SVG_WIDTH / 2;
    let text_y = EMPTY_SVG_HEIGHT / 2 + EMPTY_SVG_FONT_SIZE / 4; // approximate vertical centering
    let bg_rect = if opaque {
        format!(
            r#"<rect width="{}" height="{}" rx="{}" fill="{}"/>"#,
            EMPTY_SVG_WIDTH, EMPTY_SVG_HEIGHT, BORDER_RADIUS, theme.bg
        )
    } else {
        String::new()
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
  </style>
  {}
  <text x="{}" y="{}" fill="{}" font-size="{}" text-anchor="middle">{}</text>
</svg>"#,
        EMPTY_SVG_WIDTH,
        EMPTY_SVG_HEIGHT,
        EMPTY_SVG_WIDTH,
        EMPTY_SVG_HEIGHT,
        bg_rect,
        text_x,
        text_y,
        theme.text,
        EMPTY_SVG_FONT_SIZE,
        message
    )
}

/// Common CSS styles for SVG tiles
pub const SVG_STYLES: &str = "text { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }";
