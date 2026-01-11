mod github;
mod icons;
mod svg;
mod theme;
mod tiles;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::Path;
use tiles::{Contributions, Languages, RenderConfig, Statistics, Tile};

#[derive(Parser)]
#[command(about = "Generate GitHub stats SVGs")]
struct Args {
    /// GitHub token (https://github.com/settings/tokens/new?scopes=repo,read:user&description=GitHub%20Tiles)
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: String,

    /// Output directory
    #[arg(short, long, default_value = "assets")]
    output: String,

    /// Include private repositories
    #[arg(short, long, default_value = "false")]
    private: bool,

    /// Show username in titles
    #[arg(short, long, default_value = "false")]
    username: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = reqwest::Client::new();

    println!("Fetching GitHub data...");

    let user_data = github::fetch_user_data(&client, &args.token, args.private).await?;
    let user = user_data
        .viewer
        .context("Failed to get authenticated user data")?;

    let username = &user.login;
    let show_username = args.username;

    // Extract tile data from user
    let statistics = Statistics::from_user(&user);
    let languages = Languages::from_user(&user);
    let mut contributions = Contributions::from_user(&user, username);

    // Fetch avatars for contributions
    println!("Fetching avatars...");
    contributions.fetch_avatars(&client).await?;

    // Create output directory
    let output_path = Path::new(&args.output);
    fs::create_dir_all(output_path)?;

    // Collect all tiles
    let tiles: Vec<&dyn Tile> = vec![&statistics, &languages, &contributions];

    // Generate SVGs for all themes
    for theme in theme::ALL {
        let config = RenderConfig::new(username, show_username, theme);

        for tile in &tiles {
            let svg = svg::optimize(&tile.render(&config));
            let filename = tile.filename(theme.name);
            fs::write(output_path.join(&filename), &svg)?;
        }

        println!("Generated {} theme SVGs", theme.name);
    }

    println!("Done! SVGs saved to {}/", args.output);

    Ok(())
}
