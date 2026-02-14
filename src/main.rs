use anyhow::Context;
use clap::Parser;

use crate::frontend::PhotoBoothApplication;

mod backend;
mod config;
mod frontend;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    ///
    /// The configuration file should be in JSON format. Documentation for the
    /// config file can be found in `src/config.rs`.
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up logging
    env_logger::init();

    // Parse the config file
    log::info!("Parsing config");
    let args = Args::parse();
    let config_str = tokio::fs::read_to_string(args.config)
        .await
        .with_context(|| format!("failed to read config file at path: {}", args.config))?;
    let config: config::Config =
        serde_json::from_str(&config_str).context("failed to parse config")?;

    // Start the backend
    log::info!("Starting backend");
    let manager = backend::manager::BackendManager::from_config(&config)
        .await
        .context("failed to start backend")?;

    // Start the frontend
    log::info!("Starting application");
    iced::application(
        "Photo Booth",
        PhotoBoothApplication::update,
        PhotoBoothApplication::view,
    )
    .font(include_bytes!(
        "../assets/fonts/Noto_Color_Emoji/NotoColorEmoji-Regular.ttf"
    ))
    .font(include_bytes!(
        "../assets/fonts/Montserrat/Montserrat-Regular.ttf"
    ))
    .default_font(iced::Font::with_name("Montserrat"))
    .theme(|_| iced::Theme::custom("Custom palette".to_owned(), config.theme.into()))
    .subscription(PhotoBoothApplication::subscription)
    .run_with(|| (PhotoBoothApplication::new(manager), iced::Task::none()))
    .context("failed to run application");

    log::info!("Application exited successfully");
    Ok(())
}
