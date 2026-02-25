use anyhow::Context;
use clap::{Parser, Subcommand};
use screen_wake_lock::ScreenWakeLock;

use crate::frontend::PhotoBoothApplication;

mod backend;
mod config;
mod frontend;

#[derive(Parser, Debug)]
#[command(name = "photo-booth-v2")]
#[command(version, about = "A photo booth application", long_about = None)]
struct Cli {
    /// The command to run
    #[command(subcommand)]
    command: Option<Commands>,
    /// Path to the configuration file
    ///
    /// The configuration file should be in JSON format. Documentation for the
    /// config file can be found in `src/config.rs`.
    #[arg(short, long)]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the photo booth application
    #[command(name = "run")]
    Run,
    /// Process a given photo for printing according to the config and save the output to a file
    #[command(name = "process-photo")]
    ProcessPhoto {
        /// Path to the input photo to process
        #[arg(short, long)]
        input: String,
        /// Path to save the processed photo to
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    // Set up logging
    env_logger::init();

    // Parse the config file
    log::info!("Parsing config");
    let args = Cli::parse();
    let config_str = std::fs::read_to_string(&args.config)
        .with_context(|| format!("failed to read config file at path: {}", &args.config))?;
    let config: config::Config =
        serde_json::from_str(&config_str).context("failed to parse config")?;
    let config: &'static config::Config = Box::leak(Box::new(config)); // leak the config, since it is used for the entire duration of the program

    // Run the specified command
    match args.command.unwrap_or(Commands::Run) {
        Commands::Run => run(config),
        Commands::ProcessPhoto { input, output } => {
            let input_photo = image::open(&input)
                .with_context(|| format!("failed to open input photo at path: {}", &input))?
                .to_rgb8();
            let processed_photo = backend::manager::printer::preprocess_photo(
                config
                    .printer
                    .as_ref()
                    .ok_or(anyhow::anyhow!("no printer config"))?
                    .clone()
                    .into(),
                input_photo,
            );
            processed_photo
                .save(&output)
                .with_context(|| format!("failed to save processed photo to path: {}", &output))?;
            Ok(())
        }
    }
}

fn run(config: &'static config::Config) -> anyhow::Result<()> {
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
