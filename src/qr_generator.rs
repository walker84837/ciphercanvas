use crate::{error::Error, image_ops::save_image};
use log::{info, warn};
use miette::Result;
use qrcode::{EcLevel, QrCode, render::svg};
use std::path::PathBuf;

#[cfg(feature = "kitty_graphics")]
use crate::image_ops::load_svg;
#[cfg(feature = "kitty_graphics")]
use kitty_image::{Action, ActionPut, ActionTransmission, Command, Format, Medium, WrappedCommand};
#[cfg(feature = "kitty_graphics")]
use std::io::Write;

pub struct QrCodeOptions {
    pub ssid: String,
    pub encryption: String,
    pub password: String,
    pub output_path: Option<PathBuf>,
    pub dark_color: String,
    pub light_color: String,
    pub size: u32,
    pub format: String,
    pub overwrite: bool,
}

#[cfg(feature = "kitty_graphics")]
pub fn print_qr_code_kitty(options: &QrCodeOptions) -> Result<(), Error> {
    let contents_to_encode = format!(
        "WIFI:S:{};T:{};P:{};;",
        options.ssid,
        options.encryption.to_uppercase(),
        options.password
    );

    let qrcode = QrCode::with_error_correction_level(contents_to_encode.as_bytes(), EcLevel::H)
        .map_err(|e| Error::QrCode(format!("Failed to generate the QR code: {e}")))?;
    info!("QR code generated successfully.");

    let image_svg = qrcode
        .render()
        .min_dimensions(options.size, options.size)
        .dark_color(svg::Color(&options.dark_color))
        .light_color(svg::Color(&options.light_color))
        .build();
    info!("QR code rendered to SVG.");

    let pixmap = load_svg(image_svg.as_bytes(), options.size)?;
    let png_data = pixmap
        .encode_png()
        .map_err(|e| Error::Image(format!("Failed to encode PNG: {e}")))?;
    info!("Encoded QR code to PNG.");

    let action = Action::TransmitAndDisplay(
        ActionTransmission {
            format: Format::Png,
            medium: Medium::Direct,
            width: options.size,
            height: options.size,
            ..Default::default()
        },
        ActionPut {
            move_cursor: true,
            ..Default::default()
        },
    );

    let mut command = Command::new(action);
    command.payload = std::borrow::Cow::Borrowed(&png_data);

    let wrapped = WrappedCommand::new(command);
    let mut stdout = std::io::stdout().lock();
    wrapped
        .send_chunked(&mut stdout)
        .map_err(|e| Error::Image(format!("Failed to send to kitty: {}", e)))?;
    stdout.flush()?;
    println!();

    info!("Printed QR code to terminal using Kitty graphics protocol.");

    Ok(())
}

pub fn generate_qr_code(options: &QrCodeOptions) -> Result<(), Error> {
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
        .map_err(|e| Error::QrCode(format!("Failed to generate the QR code: {e}")))?;
    info!("QR code generated successfully.");

    let image = qrcode
        .render()
        .min_dimensions(options.size, options.size)
        .dark_color(svg::Color(&options.dark_color))
        .light_color(svg::Color(&options.light_color))
        .build();

    info!("QR code rendered to image.");

    if let Some(path) = &options.output_path {
        save_image(
            path,
            &options.format,
            &image,
            options.size,
            options.overwrite,
        )?;
    } else {
        println!("{image}");
    }
    Ok(())
}
