use crate::{error::Error, image_ops::save_image};
use log::{info, warn};
use miette::Result;
use qrcode::{EcLevel, QrCode, render::svg};
use resvg::render;
use std::path::PathBuf;
use tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree, fontdb};

#[cfg(feature = "kitty_graphics")]
use base64::{engine::general_purpose, prelude::*};
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
fn load_svg_for_kitty(contents: &[u8], size: u32) -> Result<Pixmap, Error> {
    info!("Loading SVG content with size {size}x{size}");

    let options = Options::default();
    let fontdb = fontdb::Database::new();
    let tree: Tree = Tree::from_data(contents, &options, &fontdb).map_err(|e| {
        Error::Image(format!(
            "Failed to create SVG tree from data of size {size}x{size}: {e}"
        ))
    })?;

    let mut pixmap: Pixmap =
        Pixmap::new(size, size).ok_or(Error::Image("Failed to create a new Pixmap".to_string()))?;

    render(&tree, Transform::default(), &mut pixmap.as_mut());
    info!("Rendered SVG to Pixmap");

    Ok(pixmap)
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

    let pixmap = load_svg_for_kitty(image_svg.as_bytes(), options.size)?;
    let png_data = pixmap
        .encode_png()
        .map_err(|e| Error::Image(format!("Failed to encode PNG: {e}")))?;
    info!("Encoded QR code to PNG.");

    let encoded_data = general_purpose::STANDARD.encode(&png_data);
    let chunks: Vec<&[u8]> = encoded_data.as_bytes().chunks(4096).collect();
    let num_chunks = chunks.len();

    let mut stdout = std::io::stdout().lock();

    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            // First chunk
            write!(stdout, "\x1b_Gf=100,a=T,")?;
        } else {
            // Subsequent chunks
            write!(stdout, "\x1b_G")?;
        }

        if i == num_chunks - 1 {
            // Last chunk
            write!(stdout, "m=0;")?;
        } else {
            // Not the last chunk
            write!(stdout, "m=1;")?;
        }

        stdout.write_all(chunk)?;
        write!(stdout, "\x1b\\")?;
    }
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
