use crate::error::Error;
use log::{error, info};
use resvg::render;
use std::{
    fs::File,
    io::{BufWriter, prelude::*},
    path::Path,
};
use tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree, fontdb};

const SUPPORTED_FORMATS: &[&str] = &["svg", "png"];

/// Load and render SVG content into a Pixmap of the specified size.
fn load_svg(contents: &[u8], size: u32) -> Result<Pixmap, Error> {
    info!("Loading SVG content with size {}x{}", size, size);

    let options = Options::default();
    let fontdb = fontdb::Database::new();
    let tree: Tree = Tree::from_data(contents, &options, &fontdb).map_err(|e| {
        Error::Image(format!(
            "Failed to create SVG tree from data of size {}x{}: {}",
            size, size, e
        ))
    })?;

    let mut pixmap: Pixmap =
        Pixmap::new(size, size).ok_or(Error::Image("Failed to create a new Pixmap".to_string()))?;

    render(&tree, Transform::default(), &mut pixmap.as_mut());
    info!("Rendered SVG to Pixmap");

    Ok(pixmap)
}

/// Save an image to a file. Supports both SVG and PNG output formats.
///
/// When processing a PNG image, if the requested size is small (<256px), a warning is logged.
///
/// # Usage Examples
///
/// Save an SVG image:
/// ```rust
/// use ciphercanvas::save_image;
/// let image = "<svg>...</svg>";
/// let format = "svg";
/// let size = 128;
/// let output = PathBuf::from("output.svg");
/// save_image(&output, &format, &image, size).unwrap();
/// ```
///
/// Save a PNG image:
/// ```rust
/// use ciphercanvas::save_image;
/// let image = "<svg>...</svg>";
/// let format = "png";
/// let size = 128;
/// let output = PathBuf::from("output.png");
/// save_image(&output, &format, &image, size).unwrap();
/// ```
pub fn save_image(output: &Path, format: &str, image: &str, size: u32) -> Result<(), Error> {
    info!(
        "Starting to save image with format '{}' to {:?}",
        format, output
    );

    if !SUPPORTED_FORMATS.contains(&format) {
        return Err(Error::UnsupportedFormat(format.to_string()));
    }

    let file_path = output.with_extension(format);

    match format {
        "svg" => {
            let mut writer = BufWriter::new(File::create(&file_path)?);
            writer.write_all(image.as_bytes())?;
            info!("Saved SVG image to {:?}", file_path);
        }
        "png" => {
            if size <= 256 {
                error!(
                    "Warning: Image size is {}x{}, which may result in lower quality.",
                    size, size
                );
            }
            let pixmap = load_svg(image.as_bytes(), size)?;
            pixmap
                .save_png(&file_path)
                .map_err(|e| Error::Image(e.to_string()))?;
            info!("Saved PNG image to {:?}", file_path);
        }
        _ => {
            return Err(Error::UnsupportedFormat(format.to_string()));
        }
    }

    info!("Image saved successfully to {:?}", file_path);
    Ok(())
}
