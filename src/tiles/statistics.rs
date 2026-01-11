use super::{BORDER_RADIUS, CHAR_WIDTH, FONT_SIZE, RenderConfig, SVG_STYLES, Tile};
use crate::github::User;
use crate::icons;
use crate::svg::format_number;
use log::debug;

// Layout constants
const ROW_HEIGHT: usize = 20;
const NUM_ROWS: usize = 3;
const NUM_COLS: usize = 2;
const CONTENT_HEIGHT: usize = 16;
const COL_WIDTH: usize = 170;
const ICON_OFFSET: usize = 22;
const TEXT_Y: usize = 12;
const SEPARATOR_LEN: usize = 2; // ": "

/// Statistics data extracted from GitHub user
pub struct Statistics {
    pub total_stars: u32,
    pub total_forks: u32,
    pub total_commits: u32,
    pub total_prs: u32,
    pub total_issues: u32,
    pub merged_prs: u32,
}

impl Statistics {
    /// Extract statistics from GitHub user data
    pub fn from_user(user: &User) -> Self {
        debug!("Extracting user statistics");
        let total_stars: u32 = user
            .repositories
            .nodes
            .iter()
            .filter(|r| !r.is_fork)
            .map(|r| r.stargazer_count)
            .sum();

        let total_forks: u32 = user
            .repositories
            .nodes
            .iter()
            .filter(|r| !r.is_fork)
            .map(|r| r.fork_count)
            .sum();

        let total_commits = user.contributions_collection.total_commit_contributions
            + user.contributions_collection.restricted_contributions_count;

        let stats = Self {
            total_stars,
            total_forks,
            total_commits,
            total_prs: user.pull_requests.total_count,
            total_issues: user.issues.total_count,
            merged_prs: user.merged_pull_requests.total_count,
        };

        debug!(
            "Stats: {} stars, {} forks, {} commits, {} PRs, {} issues, {} merged PRs",
            stats.total_stars,
            stats.total_forks,
            stats.total_commits,
            stats.total_prs,
            stats.total_issues,
            stats.merged_prs
        );

        stats
    }
}

impl Tile for Statistics {
    fn name(&self) -> &'static str {
        "statistics"
    }

    fn render(&self, config: &RenderConfig) -> String {
        debug!("Rendering statistics tile");
        let theme = config.theme;

        let items: [(&str, u32, &str); 6] = [
            ("Total Stars", self.total_stars, icons::STAR),
            ("Total Forks", self.total_forks, icons::FORK),
            ("Total Commits", self.total_commits, icons::COMMIT),
            ("Total PRs", self.total_prs, icons::PULL_REQUEST),
            ("Total Issues", self.total_issues, icons::ISSUE),
            ("Merged PRs", self.merged_prs, icons::CONTRIBUTION),
        ];

        let height = (NUM_ROWS - 1) * ROW_HEIGHT + CONTENT_HEIGHT;
        let mut rows = String::new();

        // Calculate max text width for each column
        let mut max_col_widths = [0usize; NUM_COLS];

        for (i, (label, value, _)) in items.iter().enumerate() {
            let col = i % NUM_COLS;
            let text_len = label.len() + SEPARATOR_LEN + format_number(*value).len();
            let item_width = ICON_OFFSET + (text_len as f64 * CHAR_WIDTH) as usize;
            max_col_widths[col] = max_col_widths[col].max(item_width);
        }

        for (i, (label, value, icon)) in items.iter().enumerate() {
            let row = i / NUM_COLS;
            let col = i % NUM_COLS;
            let x = col * COL_WIDTH;
            let y = row * ROW_HEIGHT;

            rows.push_str(&format!(
                r#"
            <g transform="translate({}, {})">
                <g fill="{}">{}</g>
                <text x="{}" y="{}" fill="{}" font-size="{}">{}: <tspan font-weight="bold">{}</tspan></text>
            </g>"#,
                x,
                y,
                theme.icon,
                icon,
                ICON_OFFSET,
                TEXT_Y,
                theme.text,
                FONT_SIZE,
                label,
                format_number(*value)
            ));
        }

        let width = COL_WIDTH + max_col_widths[1];
        let bg_rect = if config.opaque {
            format!(
                r#"<rect width="{}" height="{}" rx="{}" fill="{}"/>"#,
                width, height, BORDER_RADIUS, theme.bg
            )
        } else {
            String::new()
        };

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <style>{}</style>
  {}
  {}
</svg>"#,
            width, height, width, height, SVG_STYLES, bg_rect, rows
        )
    }
}
