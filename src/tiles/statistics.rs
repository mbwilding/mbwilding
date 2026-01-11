use super::{RenderConfig, SVG_STYLES, Tile};
use crate::github::User;
use crate::icons;
use crate::svg::format_number;

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

        Self {
            total_stars,
            total_forks,
            total_commits,
            total_prs: user.pull_requests.total_count,
            total_issues: user.issues.total_count,
            merged_prs: user.merged_pull_requests.total_count,
        }
    }
}

impl Tile for Statistics {
    fn name(&self) -> &'static str {
        "statistics"
    }

    fn render(&self, config: &RenderConfig) -> String {
        let theme = config.theme;

        let items: [(&str, u32, &str); 6] = [
            ("Total Stars", self.total_stars, icons::STAR),
            ("Total Forks", self.total_forks, icons::FORK),
            ("Total Commits", self.total_commits, icons::COMMIT),
            ("Total PRs", self.total_prs, icons::PULL_REQUEST),
            ("Total Issues", self.total_issues, icons::ISSUE),
            ("Merged PRs", self.merged_prs, icons::CONTRIBUTION),
        ];

        let row_height = 35;
        let num_rows = 3;
        let height = num_rows * row_height;
        let mut rows = String::new();

        for (i, (label, value, icon)) in items.iter().enumerate() {
            let row = i / 2;
            let col = i % 2;
            let x = col * 170;
            let y = row * row_height;

            rows.push_str(&format!(
                r#"
            <g transform="translate({}, {})">
                <g fill="{}">{}</g>
                <text x="22" y="12" fill="{}" font-size="12">{}: <tspan font-weight="bold">{}</tspan></text>
            </g>"#,
                x,
                y,
                theme.icon,
                icon,
                theme.text,
                label,
                format_number(*value)
            ));
        }

        let width = 340;
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
</svg>"#,
            width, height, width, height, SVG_STYLES, bg_rect, rows
        )
    }
}
