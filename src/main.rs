mod github;
mod icons;
mod svg;
mod theme;
mod tiles;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use std::path::Path;
use tiles::{Contributions, Languages, RenderConfig, Statistics, Tile};
use tokio::fs;

#[derive(Parser)]
#[command(about = "Generate GitHub stats SVGs")]
struct Args {
    /// GitHub token (https://github.com/settings/tokens/new?scopes=repo,read:user&description=GitHub%20Tiles)
    #[arg(long, env = "GITHUB_TOKEN")]
    token: String,

    /// Output directory
    #[arg(long, default_value = "assets")]
    output: String,

    /// Include private repositories
    #[arg(long, default_value = "false")]
    private: bool,

    /// Render opaque background
    #[arg(long, default_value = "false")]
    opaque: bool,

    /// Languages to display
    #[arg(long, default_value = "5")]
    languages: u8,

    /// Contributions to display
    #[arg(long, default_value = "10")]
    contributions: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_default();
    let filter = match rust_log.as_str() {
        "" => "warn,github_tiles=info".to_string(),
        "trace" => rust_log.to_string(),
        "debug" => format!("warn,github_tiles={}", rust_log),
        "info" | "warn" | "error" => format!("warn,github_tiles={}", rust_log),
        _ => {
            if rust_log.contains("github_tiles") {
                format!("warn,{}", rust_log)
            } else {
                format!("warn,github_tiles=info,{}", rust_log)
            }
        }
    };

    env_logger::Builder::new()
        .parse_filters(&filter)
        .format_target(false)
        .init();

    let args = Args::parse();
    let client = reqwest::Client::new();

    info!("Fetching GitHub data...");

    let user_data = github::fetch_user_data(&client, &args.token, args.private).await?;
    let user = user_data
        .viewer
        .context("Failed to get authenticated user data")?;

    let username = &user.login;
    debug!("Authenticated as: {}", username);

    // Extract tile data from user
    let statistics = Statistics::from_user(&user);
    let languages = Languages::from_user(&user, args.languages);
    let mut contributions = Contributions::from_user(&user, username, args.contributions);

    // Fetch avatars for contributions
    info!("Fetching avatars...");
    contributions.fetch_avatars(&client).await?;

    // Create output directory
    let output_path = Path::new(&args.output);
    debug!("Output directory: {}", output_path.display());
    fs::create_dir_all(output_path).await?;

    // Collect all tiles
    let tiles: Vec<&dyn Tile> = vec![&statistics, &languages, &contributions];

    // Generate SVGs for all themes
    for theme in theme::ALL {
        let config = RenderConfig::new(theme, args.opaque);

        for tile in &tiles {
            let svg = svg::optimize(&tile.render(&config));
            let filename = tile.filename(theme.name);
            debug!("Writing: {}", filename);
            fs::write(output_path.join(&filename), &svg).await?;
        }

        info!("Generated {} theme SVGs", theme.name);
    }

    info!("Done! SVGs saved to {}/", args.output);

    Ok(())
}
