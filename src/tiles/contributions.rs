use super::{RenderConfig, SVG_STYLES, Tile, empty_svg};
use crate::github::User;
use crate::icons;
use crate::svg::format_number;
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::HashMap;

/// A repository contribution entry
pub struct ContributionEntry {
    pub owner: String,
    pub repo: String,
    pub stars: u32,
    pub avatar_data: String,
}

/// Contributions data extracted from GitHub user
pub struct Contributions {
    pub repos: Vec<ContributionEntry>,
}

impl Contributions {
    /// Extract contribution data from GitHub user data
    pub fn from_user(user: &User, username: &str) -> Self {
        let mut seen: HashMap<(String, String), (u32, String)> = HashMap::new();

        for pr in &user.merged_pull_requests.nodes {
            let owner = &pr.repository.owner.login;
            if owner.to_lowercase() == username.to_lowercase() {
                continue;
            }
            let key = (owner.clone(), pr.repository.name.clone());
            seen.entry(key).or_insert((
                pr.repository.stargazer_count,
                pr.repository.owner.avatar_url.clone(),
            ));
        }

        let mut repos: Vec<_> = seen
            .into_iter()
            .map(|((owner, repo), (stars, avatar_url))| ContributionEntry {
                owner,
                repo,
                stars,
                // Will be replaced with base64 data
                avatar_data: avatar_url,
            })
            .collect();

        repos.sort_by(|a, b| b.stars.cmp(&a.stars));
        repos.truncate(10);

        Self { repos }
    }

    /// Fetch avatars and convert to base64 for embedding in SVG
    pub async fn fetch_avatars(&mut self, client: &reqwest::Client) -> Result<()> {
        for entry in &mut self.repos {
            let base64_data = match client.get(&entry.avatar_data).send().await {
                Ok(response) => {
                    if let Ok(bytes) = response.bytes().await {
                        let encoded = BASE64.encode(&bytes);
                        format!("data:image/png;base64,{}", encoded)
                    } else {
                        String::new()
                    }
                }
                Err(_) => String::new(),
            };
            entry.avatar_data = base64_data;
        }

        Ok(())
    }
}

impl Tile for Contributions {
    fn name(&self) -> &'static str {
        "contributions"
    }

    fn render(&self, config: &RenderConfig) -> String {
        let theme = config.theme;

        if self.repos.is_empty() {
            return empty_svg("No External Contributions", theme, config.opaque);
        }

        let row_height = 32;
        let avatar_size = 20;

        // Calculate the longest repo text to determine star position
        let char_width = 7.0;
        let max_repo_len = self
            .repos
            .iter()
            .map(|e| e.owner.len() + 1 + e.repo.len())
            .max()
            .unwrap_or(0);

        let text_x = avatar_size + 8;
        let star_x = text_x + (max_repo_len as f64 * char_width) as usize + 15;
        let width = star_x + 60;

        let height = self.repos.len() * row_height;
        let mut rows = String::new();

        for (i, entry) in self.repos.iter().enumerate() {
            let y = i * row_height;
            let repo_url = format!("https://github.com/{}/{}", entry.owner, entry.repo);
            rows.push_str(&format!(
                r#"<g transform="translate(0, {})">
                <clipPath id="avatar-clip-{}">
                    <circle cx="{}" cy="{}" r="{}"/>
                </clipPath>
                <a href="{}" target="_blank">
                    <image href="{}" x="0" y="0" width="{}" height="{}" clip-path="url(#avatar-clip-{})"/>
                    <text x="{}" y="14" fill="{}" font-size="12">
                        <tspan fill="{}">{}</tspan>/<tspan font-weight="600">{}</tspan>
                    </text>
                </a>
                <g transform="translate({}, 0)" fill="{}">
                    <g transform="translate(0, 4)">{}</g>
                    <text x="18" y="14" fill="{}" font-size="11">{}</text>
                </g>
            </g>"#,
                y,
                i,
                avatar_size / 2,
                avatar_size / 2,
                avatar_size / 2,
                repo_url,
                entry.avatar_data,
                avatar_size,
                avatar_size,
                i,
                text_x,
                theme.text,
                theme.icon,
                entry.owner,
                entry.repo,
                star_x,
                theme.star,
                icons::STAR,
                theme.text,
                format_number(entry.stars)
            ));
        }

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
  <style>
    {}
    a {{ text-decoration: none; }}
    a:hover text {{ text-decoration: underline; }}
  </style>
  {}
  {}
</svg>"#,
            width, height, width, height, SVG_STYLES, bg_rect, rows
        )
    }
}
