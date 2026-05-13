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
    let contents_to_encode =
        build_wifi_qr_payload(&options.ssid, &options.encryption, &options.password);

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

    let contents_to_encode =
        build_wifi_qr_payload(&options.ssid, &options.encryption, &options.password);

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

/// Build the standard Wi-Fi QR code payload string.
///
/// Format: `WIFI:S:<ssid>;T:<encryption>;P:<password>;;`
/// See: <https://github.com/zxing/zxing/wiki/Barcode-Contents#wi-fi-network-config-android-ios-11>
fn escape_wifi_value(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            ':' => out.push_str("\\:"),
            _ => out.push(c),
        }
    }
    out
}

fn build_wifi_qr_payload(ssid: &str, encryption: &str, password: &str) -> String {
    let ssid_escaped = escape_wifi_value(ssid);
    let password_escaped = escape_wifi_value(password);
    let encryption_escaped = escape_wifi_value(&encryption.to_uppercase());
    format!(
        "WIFI:S:{};T:{};P:{};;",
        ssid_escaped, encryption_escaped, password_escaped
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_qr_format_basic() {
        let payload = build_wifi_qr_payload("MyNetwork", "WPA", "secret123");
        assert_eq!(payload, "WIFI:S:MyNetwork;T:WPA;P:secret123;;");
    }

    #[test]
    fn wifi_qr_format_none_encryption() {
        let payload = build_wifi_qr_payload("GuestWifi", "None", "nopass");
        assert_eq!(payload, "WIFI:S:GuestWifi;T:NONE;P:nopass;;");
    }

    #[test]
    fn wifi_qr_format_lowercase_encryption_uppercased() {
        let payload = build_wifi_qr_payload("Home", "wpa", "password");
        assert_eq!(payload, "WIFI:S:Home;T:WPA;P:password;;");
    }

    #[test]
    fn wifi_qr_format_wep() {
        let payload = build_wifi_qr_payload("OldNetwork", "WEP", "wepkey");
        assert_eq!(payload, "WIFI:S:OldNetwork;T:WEP;P:wepkey;;");
    }

    #[test]
    fn wifi_qr_empty_ssid() {
        let payload = build_wifi_qr_payload("", "WPA", "password");
        assert_eq!(payload, "WIFI:S:;T:WPA;P:password;;");
    }

    #[test]
    fn wifi_qr_empty_password() {
        let payload = build_wifi_qr_payload("MyNetwork", "None", "");
        assert_eq!(payload, "WIFI:S:MyNetwork;T:NONE;P:;;");
    }

    #[test]
    fn wifi_qr_special_chars_in_ssid() {
        let payload = build_wifi_qr_payload("My\\Network", "WPA", "pass\\word");
        assert_eq!(payload, "WIFI:S:My\\\\Network;T:WPA;P:pass\\\\word;;");
    }
}
