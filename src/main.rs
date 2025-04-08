use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;
use log::{info, warn};
use miette::{Context, IntoDiagnostic, Result};
use qrcode::{EcLevel, QrCode, render::svg};
use std::{fmt, path::PathBuf};
use tokio::io::AsyncReadExt;

mod get_config;
mod image_ops;
mod lua_api;

use crate::{
    get_config::{get_config_int, get_config_str},
    image_ops::async_save_image,
};

/// Mature and modular CLI tool to generate QR codes and customize behavior via scripting.
///
/// # Examples
///
/// Generate a QR code with a custom configuration:
/// ```sh
/// ciphercanvas generate --ssid MyNetwork --output qrcode.svg --config ./config.toml
/// ```
///
/// Run a custom Lua script to alter tool behavior:
/// ```sh
/// ciphercanvas script --script ./customize.lua
/// ```
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// Activate verbose mode for detailed logs
    #[arg(short, long)]
    verbose: bool,

    /// Optional configuration file. If omitted, the default configuration directory is used.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Specify subcommand to execute.
    #[command(subcommand)]
    command: Option<Commands>,
}

/// List of available subcommands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate a QR code image from WiFi credentials.
    Generate {
        /// The WiFi network's SSID (name)
        #[arg(short, long)]
        ssid: Option<String>,

        /// The encryption type used (WPA, WEP, or None).
        #[arg(short, long, default_value = "wpa")]
        encryption: Encryption,

        /// The output file to export the QR code image.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run a Lua script to extend or customize the tool’s behavior.
    Script {
        /// Path to the Lua script.
        #[arg(short, long)]
        script: PathBuf,
    },
    /// Save frequently used settings in the configuration store.
    SaveSettings {
        /// Settings in TOML format to save.
        #[arg(short, long)]
        settings: String,
    },
}

/// Valid encryption types for WiFi.
#[derive(ValueEnum, Clone, Debug)]
enum Encryption {
    Wpa,
    Wep,
    None,
}

impl fmt::Display for Encryption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encryption_str = match self {
            Encryption::Wpa => "WPA",
            Encryption::Wep => "WEP",
            Encryption::None => "nopass",
        };
        write!(f, "{}", encryption_str)
    }
}

struct Consts;

impl Consts {
    const FORMAT: &'static str = "svg";
    const BACKGROUND: &'static str = "#000000";
    const FOREGROUND: &'static str = "#ffffff";
    const SIZE: u32 = 512;
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    if args.verbose {
        simple_logger::init().unwrap();
        info!("Verbose logging enabled.");
    }
    info!("Parsed arguments: {:#?}", args);

    // Determine the configuration file:
    let config_path = match &args.config {
        Some(path) if path.to_string_lossy() != "-" => path.clone(),
        _ => {
            if let Some(proj_dirs) = ProjectDirs::from("org", "winlogon", "ciphercanvas") {
                let default_path = proj_dirs.config_dir().join("config.toml");
                info!("Using default configuration file: {:?}", default_path);
                default_path
            } else {
                miette::bail!(
                    "Unable to determine the default configuration directory. Specify a config file using --config."
                );
            }
        }
    };

    // Read configuration from file or from stdin (if "-" is provided)
    let config_str = if config_path.to_string_lossy() == "-" {
        let mut input = String::new();
        tokio::io::stdin()
            .read_to_string(&mut input)
            .await
            .into_diagnostic()
            .with_context(|| "Failed to read configuration from stdin")?;
        input
    } else {
        read_config(&config_path).await?
    };
    info!("Configuration loaded successfully.");

    let toml_config: toml::Value = toml::from_str(&config_str)
        .into_diagnostic()
        .with_context(|| "Failed to parse the TOML configuration file")?;
    info!("TOML configuration parsed successfully.");

    // Process the chosen subcommand
    match args.command.unwrap_or(Commands::Generate {
        ssid: None,
        encryption: Encryption::Wpa,
        output: None,
    }) {
        Commands::Generate {
            ssid,
            encryption,
            output,
        } => {
            // Get configuration settings for QR code; if not set in TOML, use defaults.
            let export_format = get_config_str(&toml_config, "qrcode", "export", Consts::FORMAT);
            let size = get_config_int(&toml_config, "qrcode", "size", Consts::SIZE as i64) as u32;
            if size < 256 {
                warn!("Image size is lower than 256. The resulting QR code may appear cropped.");
            }
            let foreground =
                get_config_str(&toml_config, "colors", "foreground", Consts::FOREGROUND);
            let background =
                get_config_str(&toml_config, "colors", "background", Consts::FOREGROUND);
            let ssid = ssid.unwrap_or_else(|| {
                toml_config
                    .get("wifi")
                    .and_then(|w| w.get("ssid"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("default_ssid")
                    .to_string()
            });

            let password = toml_config
                .get("qrcode")
                .and_then(|q| q.get("password"))
                .and_then(|p| p.as_str())
                .ok_or(miette::miette!("QR code password missing in configuration"))?;
            info!("Password retrieved from configuration.");

            let contents_to_encode = format!(
                "WIFI:S:{};T:{};P:{};;",
                ssid,
                encryption.to_string().to_uppercase(),
                password
            );

            let qrcode =
                QrCode::with_error_correction_level(contents_to_encode.as_bytes(), EcLevel::H)
                    .into_diagnostic()
                    .with_context(|| "Failed to generate the QR code")?;
            info!("QR code generated successfully.");

            let image = qrcode
                .render()
                .min_dimensions(size, size)
                .dark_color(svg::Color(&foreground))
                .light_color(svg::Color(&background))
                .build();
            info!("QR code rendered to image.");

            if let Some(output_path) = output {
                async_save_image(output_path, export_format.into(), image, size)
                    .await
                    .map_err(|e| miette::miette!(e))?;
                info!("Image saved successfully.");
            } else {
                println!("{}", image);
                info!("Image output to stdout.");
            }
        }
        Commands::Script { script } => {
            info!("Executing Lua script: {:?}", script);
            lua_api::execute_script(script)
                .await
                .with_context(|| "Error executing Lua script")?;
            info!("Lua script executed successfully.");
        }
        Commands::SaveSettings { settings } => {
            if let Some(proj_dirs) = ProjectDirs::from("org", "winlogon", "ciphercanvas") {
                let config_dir = proj_dirs.config_dir();
                let settings_path = config_dir.join("settings.toml");
                tokio::fs::write(&settings_path, settings)
                    .await
                    .into_diagnostic()
                    .with_context(|| format!("Failed to save settings to {:?}", settings_path))?;
                info!("Settings saved successfully to {:?}", settings_path);
            } else {
                miette::bail!(
                    "Unable to determine default configuration directory to save settings."
                );
            }
        }
    }

    Ok(())
}

/// Reads the configuration file asynchronously from the given path.
async fn read_config(config_path: &PathBuf) -> Result<String> {
    info!("Reading configuration file from {:?}", config_path);
    let config_str = tokio::fs::read_to_string(config_path)
        .await
        .into_diagnostic()
        .with_context(|| format!("Failed to read configuration file from {:?}", config_path))?;
    info!(
        "Configuration file read successfully from {:?}",
        config_path
    );
    Ok(config_str)
}
