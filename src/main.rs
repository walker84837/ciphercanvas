use clap::{Parser, Subcommand, ValueEnum};
use log::info;
use std::{fmt, path::PathBuf};

mod error;
mod image_ops;
mod qr_generator;

use qr_generator::QrCodeOptions;

/// Mature and modular CLI tool to generate QR codes.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// Activate verbose mode for detailed logs
    #[arg(short, long)]
    verbose: bool,

    /// Specify subcommand to execute.
    #[command(subcommand)]
    command: Option<Commands>,
}

/// List of available subcommands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate a QR code image from Wi-Fi credentials.
    Generate {
        /// The Wi-Fi network's SSID (name)
        #[arg(short, long)]
        ssid: String,

        /// The encryption type used (WPA, WEP, or None).
        #[arg(short, long, default_value = "wpa")]
        encryption: Encryption,

        /// The output file to export the QR code image.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// The Wi-Fi network's password.
        #[arg(short, long)]
        password: String,

        /// The size of the QR code image (e.g., 512).
        #[arg(long, default_value_t = 512)]
        size: u32,

        /// The output format of the image (e.g., "svg", "png").
        #[arg(long, default_value = "svg")]
        format: String,

        /// The foreground color of the QR code (e.g., "#000000").
        #[arg(long, default_value = "#000000")]
        foreground: String,

        /// The background color of the QR code (e.g., "#ffffff")]
        #[arg(long, default_value = "#ffffff")]
        background: String,

        /// Overwrite existing files without prompt.
        #[arg(long, default_value_t = false)]
        overwrite: bool,
    },
}

/// Valid encryption types for Wi-Fi.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
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
        write!(f, "{encryption_str}")
    }
}

fn main() -> Result<(), error::Error> {
    let args = CliArgs::parse();

    if args.verbose {
        simple_logger::init().unwrap();
        info!("Verbose logging enabled.");
    }
    info!("Parsed arguments: {args:#?}");

    match args.command {
        Some(Commands::Generate {
            ssid,
            encryption,
            output,
            password,
            size,
            format,
            foreground,
            background,
            overwrite,
        }) => {
            let options = QrCodeOptions {
                ssid,
                encryption: encryption.to_string(),
                password,
                output_path: output,
                dark_color: foreground.clone(),
                light_color: background.clone(),
                size,
                format: format.clone(),
                overwrite,
            };
            qr_generator::generate_qr_code(options)?;
        }
        None => {}
    }

    Ok(())
}
