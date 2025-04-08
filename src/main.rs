use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use log::{info, warn};
use qrcode::{EcLevel, QrCode, render::svg};
use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

mod get_config;
mod image_ops;

use crate::{
    get_config::{get_config_int, get_config_str},
    image_ops::save_image,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, help = "The WiFi network's SSID (its name)")]
    ssid: String,

    #[arg(
        short,
        long,
        default_value = "wpa",
        help = "The encryption of the WiFi network."
    )]
    encryption: Encryption,

    #[arg(short, long, help = "The TOML configuration file.")]
    config: PathBuf,

    #[arg(short, help = "The output file to export the QR code to.")]
    output: PathBuf,

    #[arg(short, help = "Enable verbose mode and start logging")]
    verbose: bool,
}

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

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        simple_logger::init().unwrap();
        info!("Logger initialized");
    }

    info!("Parsed arguments: {:#?}", args);

    let config_str = read_config(&args.config)?;
    info!("Configuration file loaded successfully");

    let toml_config: toml::Value =
        toml::from_str(&config_str).context("Failed to parse the TOML configuration file")?;
    info!("TOML configuration parsed successfully");

    let export_format = get_config_str(&toml_config, "qrcode", "export", Consts::FORMAT);
    let size = get_config_int(&toml_config, "qrcode", "size", Consts::SIZE as i64) as u32;

    if size < 256 {
        warn!("The image size is lower than 256. The resulting QR code may look cropped.");
    }

    let foreground = get_config_str(&toml_config, "colors", "foreground", Consts::FOREGROUND);
    let background = get_config_str(&toml_config, "colors", "background", Consts::BACKGROUND);

    let password = toml_config["qrcode"]["password"]
        .as_str()
        .context("Failed to get the password from the configuration file")?;
    info!("Password retrieved from the configuration file");

    let contents_to_encode = format!(
        "WIFI:S:{};T:{};P:{};;",
        args.ssid,
        args.encryption.to_string().to_uppercase(),
        password
    );

    let qrcode = QrCode::with_error_correction_level(contents_to_encode.as_bytes(), EcLevel::H)
        .context("Failed to generate the QR code")?;
    info!("QR code generated successfully");

    let image = qrcode
        .render()
        .min_dimensions(size, size)
        .dark_color(svg::Color(&foreground))
        .light_color(svg::Color(&background))
        .build();
    info!("QR code rendered to image");

    save_image(&args.output, &export_format, &image, size)?;
    info!("Image saved successfully to {:?}", args.output);

    Ok(())
}

fn read_config(config_path: &PathBuf) -> Result<String> {
    info!("Reading configuration file from {:?}", config_path);

    let f = File::open(config_path)
        .with_context(|| format!("Failed to open config file '{:?}'", config_path))?;
    info!("Configuration file '{:?}' opened successfully", config_path);

    let mut reader = BufReader::new(f);
    let mut config_str = String::new();

    reader
        .read_to_string(&mut config_str)
        .context("Failed to read the config file")?;
    info!("Configuration file read successfully");

    Ok(config_str)
}
