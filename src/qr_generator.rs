use crate::error::Error;
use log::{info, warn};
use miette::Result;
use qrcode::{EcLevel, QrCode, render::svg};
use std::path::PathBuf;

use crate::image_ops::save_image;

pub struct QrCodeOptions {
    pub ssid: String,
    pub encryption: String,
    pub password: String,
    pub output_path: String,
    pub dark_color: String,
    pub light_color: String,
    pub size: u32,
    pub format: String,
}

pub fn generate_qr_code(options: QrCodeOptions) -> Result<(), Error> {
    if options.size < 256 {
        warn!("Image size is lower than 256. The resulting QR code may appear cropped.");
    }

    let contents_to_encode = format!(
        "WIFI:S:{};T:{};P:{};;",
        options.ssid,
        options.encryption.to_uppercase(),
        options.password
    );

    let qrcode = QrCode::with_error_correction_level(contents_to_encode.as_bytes(), EcLevel::H)
        .map_err(|e| Error::QrCode(format!("Failed to generate the QR code: {}", e)))?;
    info!("QR code generated successfully.");

    let image = qrcode
        .render()
        .min_dimensions(options.size, options.size)
        .dark_color(svg::Color(&options.dark_color))
        .light_color(svg::Color(&options.light_color))
        .build();
    info!("QR code rendered to image.");

    if !options.output_path.is_empty() {
        save_image(
            &PathBuf::from(options.output_path),
            &options.format,
            &image,
            options.size,
        )?;
    } else {
        println!("{}", image);
        info!("Image output to stdout.");
    }
    Ok(())
}
