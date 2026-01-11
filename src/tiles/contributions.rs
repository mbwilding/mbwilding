use super::{
    BORDER_RADIUS, CHAR_WIDTH, FONT_SIZE, FONT_SIZE_SMALL, RenderConfig, SVG_STYLES, Tile,
    empty_svg,
};
use crate::github::User;
use crate::icons;
use crate::svg::format_number;
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures::future::join_all;
use log::debug;
use std::collections::HashMap;

// Layout constants
const MAX_CONTRIBUTIONS: usize = 10;
const ROW_HEIGHT: usize = 24;
const AVATAR_SIZE: usize = 20;
const AVATAR_TEXT_GAP: usize = 8;
const REPO_STAR_GAP: usize = 15;
const STAR_AREA_WIDTH: usize = 60;
const TEXT_Y: usize = 14;
const STAR_ICON_X_OFFSET: usize = 18;
const STAR_ICON_Y_OFFSET: usize = 4;

/// A repository contribution entry
pub struct ContributionEntry {
    pub owner: String,
    pub repo: String,
    pub stars: u32,
    pub avatar_data: Option<String>,
}

/// Contributions data extracted from GitHub user
pub struct Contributions {
    pub repos: Vec<ContributionEntry>,
}

impl Contributions {
    /// Extract contribution data from GitHub user data
    pub fn from_user(user: &User, username: &str) -> Self {
        debug!("Extracting contributions for user: {}", username);
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
                avatar_data: Some(avatar_url),
            })
            .collect();

        repos.sort_by(|a, b| b.stars.cmp(&a.stars));
        repos.truncate(MAX_CONTRIBUTIONS);

        debug!("Found {} external contributions", repos.len());
        for entry in &repos {
            debug!(
                "Contribution: {}/{} ({} stars)",
                entry.owner, entry.repo, entry.stars
            );
        }

        Self { repos }
    }

    /// Fetch avatars and convert to base64 for embedding in SVG
    pub async fn fetch_avatars(&mut self, client: &reqwest::Client) -> Result<()> {
        debug!("Fetching {} avatars", self.repos.len());
        let futures: Vec<_> = self
            .repos
            .iter()
            .map(|entry| {
                let avatar_url = entry.avatar_data.clone();
                let client = client.clone();
                let owner = entry.owner.clone();
                async move {
                    let url = avatar_url?;
                    debug!("Fetching avatar for {}", owner);
                    let response = client.get(&url).send().await.ok()?;
                    let bytes = response.bytes().await.ok()?;
                    let encoded = BASE64.encode(&bytes);
                    Some(format!("data:image/png;base64,{}", encoded))
                }
            })
            .collect();

        let results = join_all(futures).await;

        let mut success_count = 0;
        for (entry, result) in self.repos.iter_mut().zip(results) {
            if result.is_some() {
                success_count += 1;
            }
            entry.avatar_data = result;
        }
        debug!(
            "Successfully fetched {}/{} avatars",
            success_count,
            self.repos.len()
        );

        Ok(())
    }
}

impl Tile for Contributions {
    fn name(&self) -> &'static str {
        "contributions"
    }

    fn render(&self, config: &RenderConfig) -> String {
        debug!("Rendering contributions tile");
        let theme = config.theme;

        if self.repos.is_empty() {
            return empty_svg("No External Contributions", theme, config.opaque);
        }

        let content_height = AVATAR_SIZE;

        // Calculate the longest repo text to determine star position
        let max_repo_len = self
            .repos
            .iter()
            .map(|e| e.owner.len() + 1 + e.repo.len())
            .max()
            .unwrap_or(0);

        let text_x = AVATAR_SIZE + AVATAR_TEXT_GAP;
        let star_x = text_x + (max_repo_len as f64 * CHAR_WIDTH) as usize + REPO_STAR_GAP;
        let width = star_x + STAR_AREA_WIDTH;

        let height = (self.repos.len() - 1) * ROW_HEIGHT + content_height;
        let mut rows = String::new();

        let avatar_radius = AVATAR_SIZE / 2;
        for (i, entry) in self.repos.iter().enumerate() {
            let y = i * ROW_HEIGHT;
            rows.push_str(&format!(
                r#"<g transform="translate(0, {})">
                <clipPath id="avatar-clip-{}">
                    <circle cx="{}" cy="{}" r="{}"/>
                </clipPath>
                <image href="{}" x="0" y="0" width="{}" height="{}" clip-path="url(#avatar-clip-{})"/>
                <text x="{}" y="{}" fill="{}" font-size="{}">
                    <tspan fill="{}">{}</tspan>/<tspan font-weight="600">{}</tspan>
                </text>
                <g transform="translate({}, 0)" fill="{}">
                    <g transform="translate(0, {})">{}</g>
                    <text x="{}" y="{}" fill="{}" font-size="{}">{}</text>
                </g>
            </g>"#,
                y,
                i,
                avatar_radius,
                avatar_radius,
                avatar_radius,
                entry.avatar_data.as_deref().unwrap_or(""),
                AVATAR_SIZE,
                AVATAR_SIZE,
                i,
                text_x,
                TEXT_Y,
                theme.text,
                FONT_SIZE,
                theme.icon,
                entry.owner,
                entry.repo,
                star_x,
                theme.star,
                STAR_ICON_Y_OFFSET,
                icons::STAR,
                STAR_ICON_X_OFFSET,
                TEXT_Y,
                theme.text,
                FONT_SIZE_SMALL,
                format_number(entry.stars)
            ));
        }

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
