use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "github-stats")]
#[command(about = "Generate GitHub stats SVGs for light and dark mode")]
struct Args {
    /// GitHub token (requires read:user and repo scopes)
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: String,

    /// Output directory
    #[arg(short, long, default_value = "assets")]
    output: String,

    /// Include private repositories (public only by default)
    #[arg(long, default_value = "false")]
    private: bool,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct UserData {
    viewer: Option<User>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    login: String,
    contributions_collection: ContributionsCollection,
    repositories: RepositoryConnection,
    pull_requests: PullRequestConnection,
    issues: IssueConnection,
    merged_pull_requests: MergedPullRequestConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributionsCollection {
    total_commit_contributions: u32,
    restricted_contributions_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryConnection {
    nodes: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Repository {
    stargazer_count: u32,
    fork_count: u32,
    is_fork: bool,
    languages: Option<LanguageConnection>,
}

#[derive(Debug, Deserialize)]
struct LanguageConnection {
    edges: Vec<LanguageEdge>,
}

#[derive(Debug, Deserialize)]
struct LanguageEdge {
    size: u64,
    node: Language,
}

#[derive(Debug, Deserialize)]
struct Language {
    name: String,
    color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRequestConnection {
    total_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueConnection {
    total_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergedPullRequestConnection {
    total_count: u32,
    nodes: Vec<MergedPullRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergedPullRequest {
    repository: PullRequestRepo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRequestRepo {
    name: String,
    owner: RepoOwner,
    stargazer_count: u32,
}

#[derive(Debug, Deserialize)]
struct RepoOwner {
    login: String,
}

#[derive(Clone, Copy)]
struct Theme {
    name: &'static str,
    bg: &'static str,
    title: &'static str,
    text: &'static str,
    icon: &'static str,
}

const LIGHT_THEME: Theme = Theme {
    name: "light",
    bg: "#ffffff",
    title: "#0366d6",
    text: "#333333",
    icon: "#586069",
};

const DARK_THEME: Theme = Theme {
    name: "dark",
    bg: "#0d1117",
    title: "#58a6ff",
    text: "#c9d1d9",
    icon: "#8b949e",
};

struct Stats {
    total_stars: u32,
    total_forks: u32,
    total_commits: u32,
    total_prs: u32,
    total_issues: u32,
    contributed_to: u32,
}

struct LanguageStats {
    languages: Vec<(String, u64, String)>, // name, bytes, color
    total_bytes: u64,
}

struct ContribStats {
    repos: Vec<(String, String, u32)>, // owner, name, stars
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = reqwest::Client::new();

    println!("Fetching GitHub data...");

    let data = fetch_github_data(&client, &args.token, args.private).await?;

    let user = data
        .viewer
        .context("Failed to get authenticated user data")?;

    let username = &user.login;
    let stats = extract_stats(&user);
    let languages = extract_languages(&user);
    let contribs = extract_contribs(&user, username);

    // Create output directory
    let output_path = Path::new(&args.output);
    fs::create_dir_all(output_path)?;

    // Generate SVGs for both themes
    for theme in [LIGHT_THEME, DARK_THEME] {
        let stats_svg = generate_stats_svg(&stats, username, theme);
        let langs_svg = generate_languages_svg(&languages, username, theme);
        let contribs_svg = generate_contribs_svg(&contribs, username, theme);

        fs::write(
            output_path.join(format!("stats_{}.svg", theme.name)),
            stats_svg,
        )?;
        fs::write(
            output_path.join(format!("languages_{}.svg", theme.name)),
            langs_svg,
        )?;
        fs::write(
            output_path.join(format!("contribs_{}.svg", theme.name)),
            contribs_svg,
        )?;

        println!("Generated {} theme SVGs", theme.name);
    }

    println!("Done! SVGs saved to {}/", args.output);

    Ok(())
}

async fn fetch_github_data(
    client: &reqwest::Client,
    token: &str,
    include_private: bool,
) -> Result<UserData> {
    let privacy = if include_private { "" } else { ", privacy: PUBLIC" };
    let query = format!(r#"
        query {{
            viewer {{
                login
                contributionsCollection {{
                    totalCommitContributions
                    restrictedContributionsCount
                }}
                repositories(first: 100, ownerAffiliations: OWNER{}, orderBy: {{field: STARGAZERS, direction: DESC}}) {{
                    nodes {{
                        stargazerCount
                        forkCount
                        isFork
                        languages(first: 10, orderBy: {{field: SIZE, direction: DESC}}) {{
                            edges {{
                                size
                                node {{
                                    name
                                    color
                                }}
                            }}
                        }}
                    }}
                }}
                pullRequests(first: 1) {{
                    totalCount
                }}
                issues(first: 1) {{
                    totalCount
                }}
                mergedPullRequests: pullRequests(first: 100, states: MERGED, orderBy: {{field: CREATED_AT, direction: DESC}}) {{
                    totalCount
                    nodes {{
                        repository {{
                            name
                            owner {{
                                login
                            }}
                            stargazerCount
                        }}
                    }}
                }}
            }}
        }}
    "#, privacy);

    let body = serde_json::json!({
        "query": query
    });

    let response: GraphQLResponse<UserData> = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "github-stats-generator")
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(errors) = response.errors {
        let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        anyhow::bail!("GraphQL errors: {}", messages.join(", "));
    }

    response.data.context("No data in response")
}

fn extract_stats(user: &User) -> Stats {
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

    Stats {
        total_stars,
        total_forks,
        total_commits,
        total_prs: user.pull_requests.total_count,
        total_issues: user.issues.total_count,
        contributed_to: user.merged_pull_requests.total_count,
    }
}

fn extract_languages(user: &User) -> LanguageStats {
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
        .map(|(name, (bytes, color))| (name, bytes, color))
        .collect();

    languages.sort_by(|a, b| b.1.cmp(&a.1));

    let total_bytes: u64 = languages.iter().map(|(_, b, _)| b).sum();

    LanguageStats {
        languages,
        total_bytes,
    }
}

fn extract_contribs(user: &User, username: &str) -> ContribStats {
    // Collect unique repos from merged PRs, excluding user's own repos
    let mut seen: HashMap<(String, String), u32> = HashMap::new();

    for pr in &user.merged_pull_requests.nodes {
        let owner = &pr.repository.owner.login;
        if owner.to_lowercase() == username.to_lowercase() {
            continue;
        }
        let key = (owner.clone(), pr.repository.name.clone());
        seen.entry(key)
            .or_insert(pr.repository.stargazer_count);
    }

    // Sort by stars descending
    let mut repos: Vec<_> = seen
        .into_iter()
        .map(|((owner, name), stars)| (owner, name, stars))
        .collect();
    repos.sort_by(|a, b| b.2.cmp(&a.2));

    // Take top 10
    repos.truncate(10);

    ContribStats { repos }
}

fn generate_stats_svg(stats: &Stats, name: &str, theme: Theme) -> String {
    let items = [
        ("stars", "Total Stars", stats.total_stars, star_icon()),
        ("forks", "Total Forks", stats.total_forks, fork_icon()),
        ("commits", "Total Commits", stats.total_commits, commit_icon()),
        ("prs", "Total PRs", stats.total_prs, pr_icon()),
        ("issues", "Total Issues", stats.total_issues, issue_icon()),
        ("contribs", "Merged PRs", stats.contributed_to, contrib_icon()),
    ];

    let height = 180;
    let mut rows = String::new();

    for (i, (_, label, value, icon)) in items.iter().enumerate() {
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
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
  </style>
  <rect width="350" height="{}" rx="4.5" fill="{}"/>
  <text x="25" y="35" fill="{}" font-size="16" font-weight="600">{}'s Statistics</text>
  {}
</svg>"#,
        height,
        height,
        height,
        theme.bg,
        theme.title,
        name,
        rows
    )
}

fn generate_languages_svg(langs: &LanguageStats, name: &str, theme: Theme) -> String {
    let top_langs: Vec<_> = langs.languages.iter().take(8).collect();

    if top_langs.is_empty() {
        return generate_empty_svg("No Languages Found", theme);
    }

    // Calculate percentages
    let lang_data: Vec<_> = top_langs
        .iter()
        .map(|(name, bytes, color)| {
            let pct = (*bytes as f64 / langs.total_bytes as f64) * 100.0;
            (name.as_str(), pct, color.as_str())
        })
        .collect();

    // Generate donut chart - positioned on the right
    let cx = 80.0;
    let cy = 80.0;
    let r = 55.0;
    let inner_r = 32.0;

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

    // Generate legend - positioned on the left
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
    let donut_height = 50 + 160 + 15; // title area + donut + padding
    let height = legend_height.max(donut_height);

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="350" height="{}" viewBox="0 0 350 {}">
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
  </style>
  <rect width="350" height="{}" rx="4.5" fill="{}"/>
  <text x="25" y="35" fill="{}" font-size="16" font-weight="600">{}'s Languages</text>
  {}
  <g transform="translate(190, 50)">
    {}
  </g>
</svg>"#,
        height,
        height,
        height,
        theme.bg,
        theme.title,
        name,
        legend,
        paths
    )
}

fn generate_contribs_svg(contribs: &ContribStats, name: &str, theme: Theme) -> String {
    if contribs.repos.is_empty() {
        return generate_empty_svg("No External Contributions", theme);
    }

    let row_height = 28;
    let title_height = 50;
    let padding = 15;

    // Calculate the longest repo text to determine star position
    // Approximate character width: ~7px for 12px font size
    let char_width = 7.0;
    let max_repo_len = contribs
        .repos
        .iter()
        .map(|(owner, repo, _)| owner.len() + 1 + repo.len()) // +1 for "/"
        .max()
        .unwrap_or(0);

    // Base offset: 25 (left margin) + 22 (icon + gap) + text width + 15 (gap before stars)
    let star_x = 25 + 22 + (max_repo_len as f64 * char_width) as usize + 15;
    let width = star_x + 60; // 60 for star icon + count

    let height = title_height + contribs.repos.len() * row_height + padding;
    let mut rows = String::new();

    for (i, (owner, repo_name, stars)) in contribs.repos.iter().enumerate() {
        let y = title_height + i * row_height;
        rows.push_str(&format!(
            r#"<g transform="translate(25, {})">
                <g fill="{}">{}</g>
                <text x="22" y="12" fill="{}" font-size="12">
                    <tspan fill="{}">{}</tspan>/<tspan font-weight="600">{}</tspan>
                </text>
                <g transform="translate({}, 0)" fill="{}">
                    {}
                    <text x="18" y="12" fill="{}" font-size="11">{}</text>
                </g>
            </g>"#,
            y,
            theme.icon,
            repo_icon(),
            theme.text,
            theme.icon,
            owner,
            repo_name,
            star_x - 25, // relative to the group's x=25
            theme.icon,
            star_icon(),
            theme.text,
            format_number(*stars)
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
  </style>
  <rect width="{}" height="{}" rx="4.5" fill="{}"/>
  <text x="25" y="35" fill="{}" font-size="16" font-weight="600">{}'s Contributions</text>
  {}
</svg>"#,
        width,
        height,
        width,
        height,
        width,
        height,
        theme.bg,
        theme.title,
        name,
        rows
    )
}

fn generate_empty_svg(message: &str, theme: Theme) -> String {
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

fn format_number(n: u32) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

// SVG Icons (from Octicons)
fn star_icon() -> &'static str {
    r#"<path d="M8 .25a.75.75 0 01.673.418l1.882 3.815 4.21.612a.75.75 0 01.416 1.279l-3.046 2.97.719 4.192a.75.75 0 01-1.088.791L8 12.347l-3.766 1.98a.75.75 0 01-1.088-.79l.72-4.194L.818 6.374a.75.75 0 01.416-1.28l4.21-.611L7.327.668A.75.75 0 018 .25z" transform="scale(0.875)"/>"#
}

fn fork_icon() -> &'static str {
    r#"<path d="M5 3.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm0 2.122a2.25 2.25 0 10-1.5 0v.878A2.25 2.25 0 005.75 8.5h1.5v2.128a2.251 2.251 0 101.5 0V8.5h1.5a2.25 2.25 0 002.25-2.25v-.878a2.25 2.25 0 10-1.5 0v.878a.75.75 0 01-.75.75h-4.5A.75.75 0 015 6.25v-.878zm3.75 7.378a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm3-8.75a.75.75 0 100-1.5.75.75 0 000 1.5z" transform="scale(0.875)"/>"#
}

fn commit_icon() -> &'static str {
    r#"<path d="M11.93 8.5a4.002 4.002 0 01-7.86 0H.75a.75.75 0 010-1.5h3.32a4.002 4.002 0 017.86 0h3.32a.75.75 0 010 1.5h-3.32zM8 10.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z" transform="scale(0.875)"/>"#
}

fn pr_icon() -> &'static str {
    r#"<path d="M7.177 3.073L9.573.677A.25.25 0 0110 .854v4.792a.25.25 0 01-.427.177L7.177 3.427a.25.25 0 010-.354zM3.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 113 2.122v5.256a2.251 2.251 0 11-1.5 0V5.372A2.25 2.25 0 011.5 3.25zM11 2.5h-1V4h1a1 1 0 011 1v5.628a2.251 2.251 0 101.5 0V5A2.5 2.5 0 0011 2.5zm1 10.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0zM3.75 12a.75.75 0 100 1.5.75.75 0 000-1.5z" transform="scale(0.875)"/>"#
}

fn issue_icon() -> &'static str {
    r#"<path d="M8 9.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/><path fill-rule="evenodd" d="M8 0a8 8 0 100 16A8 8 0 008 0zM1.5 8a6.5 6.5 0 1113 0 6.5 6.5 0 01-13 0z" transform="scale(0.875)"/>"#
}

fn contrib_icon() -> &'static str {
    r#"<path d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1v9h-8a2.5 2.5 0 00-.732.107V2.5a1 1 0 011-1h7.732zM5 12.25v3.25a.25.25 0 00.4.2l1.45-1.087a.25.25 0 01.3 0L8.6 15.7a.25.25 0 00.4-.2v-3.25a.25.25 0 00-.25-.25h-3.5a.25.25 0 00-.25.25z" transform="scale(0.875)"/>"#
}

fn repo_icon() -> &'static str {
    r#"<path d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1v9h-8a2.5 2.5 0 00-.732.107V2.5a1 1 0 011-1h7.732z" transform="scale(0.875)"/>"#
}
