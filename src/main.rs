use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use log::info;
use std::{fmt, path::PathBuf};

mod error;
mod image_ops;
mod qr_generator;

use qr_generator::QrCodeOptions;

/// Validate a hex color string (e.g. "#000000" or "#ffffff").
fn validate_hex_color(color: &str) -> Result<(), String> {
    if !color.starts_with('#') {
        return Err(format!("Color must start with '#': {color}"));
    }
    let hex = &color[1..];
    if hex.len() != 6 {
        return Err(format!(
            "Color must be 6 hex digits after '#': {color} (got {} digits)",
            hex.len()
        ));
    }
    if u32::from_str_radix(hex, 16).is_err() {
        return Err(format!("Color contains invalid hex digits: {color}"));
    }
    Ok(())
}

/// Helper: generate QR code to file, or display in terminal (if kitty_graphics feature enabled and no output path).
#[cfg(feature = "kitty_graphics")]
fn generate_or_display_qr(options: &QrCodeOptions) -> Result<(), error::Error> {
    if options.output_path.is_none() {
        return qr_generator::print_qr_code_kitty(options);
    }
    qr_generator::generate_qr_code(options)
}

#[cfg(not(feature = "kitty_graphics"))]
fn generate_or_display_qr(options: &QrCodeOptions) -> Result<(), error::Error> {
    qr_generator::generate_qr_code(options)
}

/// Mature and modular CLI tool to generate QR codes.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Mature and modular CLI tool to generate QR codes.\n\nFor more information and to report issues, visit: https://github.com/walker84837/ciphercanvas-rs"
)]
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
    #[command(
        after_help = "Examples:\n  ciphercanvas generate --ssid MyWifi --password-file ./wifi_pass.txt --output wifi_qr.png\n  ciphercanvas generate --ssid MyGuestWifi --encryption None --output guest_qr.svg\n  echo \"mysecretpassword\" | ciphercanvas generate --ssid MySecureWifi --output secure_qr.png\n  ciphercanvas generate --ssid MyHomeWifi --output home_qr.png (will prompt for password)"
    )]
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

        /// Read the Wi-Fi network's password from the specified file.
        /// If not provided, the password will be read from stdin.
        #[arg(long)]
        password_file: Option<PathBuf>,

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

// Helper function to read password from file or stdin
fn get_password(password_file: Option<PathBuf>) -> Result<String> {
    if let Some(path) = password_file {
        std::fs::read_to_string(&path)
            .with_context(|| format!("Could not read password from file: {}", path.display()))
    } else {
        rpassword::read_password().context("Could not read password from stdin.")
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
            password_file,
            size,
            format,
            foreground,
            background,
            overwrite,
        }) => {
            let password = get_password(password_file)
                .map_err(error::Error::Anyhow)?
                .trim_end()
                .to_string();

            validate_hex_color(&foreground).map_err(error::Error::InvalidColor)?;
            validate_hex_color(&background).map_err(error::Error::InvalidColor)?;

            let options = QrCodeOptions {
                ssid,
                encryption: encryption.to_string(),
                password,
                output_path: output.clone(),
                dark_color: foreground,
                light_color: background,
                size,
                format,
                overwrite,
            };

            generate_or_display_qr(&options)?;

            if let Some(path) = options.output_path {
                println!(
                    "QR code successfully generated and saved to \"{}\"",
                    path.display()
                );
            }
        }
        None => {}
    }

    Ok(())
}
