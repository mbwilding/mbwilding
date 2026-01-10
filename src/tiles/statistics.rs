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
        let title = config.title("Statistics");

        let items: [(&str, u32, &str); 6] = [
            ("Total Stars", self.total_stars, icons::star()),
            ("Total Forks", self.total_forks, icons::fork()),
            ("Total Commits", self.total_commits, icons::commit()),
            ("Total PRs", self.total_prs, icons::pull_request()),
            ("Total Issues", self.total_issues, icons::issue()),
            ("Merged PRs", self.merged_prs, icons::contribution()),
        ];

        let height = 180;
        let mut rows = String::new();

        for (i, (label, value, icon)) in items.iter().enumerate() {
            let row = i / 2;
            let col = i % 2;
            let x = 25 + col * 170;
            let y = 55 + row * 35;

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

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="350" height="{}" viewBox="0 0 350 {}">
  <style>{}</style>
  <rect width="350" height="{}" rx="4.5" fill="{}"/>
  <text x="25" y="35" fill="{}" font-size="16" font-weight="600">{}</text>
  {}
</svg>"#,
            height, height, SVG_STYLES, height, theme.bg, theme.title, title, rows
        )
    }
}
