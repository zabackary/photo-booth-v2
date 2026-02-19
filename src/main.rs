use anyhow::Context;
use clap::Parser;
use screen_wake_lock::ScreenWakeLock;

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

fn main() -> anyhow::Result<()> {
    // Set up logging
    env_logger::init();

    // Parse the config file
    log::info!("Parsing config");
    let args = Args::parse();
    let config_str = std::fs::read_to_string(&args.config)
        .with_context(|| format!("failed to read config file at path: {}", &args.config))?;
    let config: config::Config =
        serde_json::from_str(&config_str).context("failed to parse config")?;
    let config: &'static config::Config = Box::leak(Box::new(config)); // leak the config, since it is used for the entire duration of the program

    let palette: iced::theme::Palette = config.theme.into();

    // Start the frontend
    log::info!("Starting application");
    let wake_lock = ScreenWakeLock::acquire("photo booth is running");
    iced::application::timed(
        move || PhotoBoothApplication::new(config),
        PhotoBoothApplication::update,
        PhotoBoothApplication::subscription,
        PhotoBoothApplication::view,
    )
    .title("photo-booth-v2")
    .font(include_bytes!(
        "../assets/fonts/Noto_Color_Emoji/NotoColorEmoji-Regular.ttf"
    ))
    .font(include_bytes!(
        "../assets/fonts/Montserrat/Montserrat-Regular.ttf"
    ))
    .default_font(iced::Font::with_name("Montserrat"))
    .theme(iced::Theme::custom("Custom palette".to_owned(), palette))
    .run()
    .context("application exited with an error")?;

    std::mem::drop(wake_lock); // release the wake lock
    log::info!("Application exited successfully");
    Ok(())
}
