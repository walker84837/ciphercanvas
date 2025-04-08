use anyhow::{Context, Result, bail};
use log::{error, info};
use resvg::render;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};
use tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree, fontdb};

/// Load and render SVG content into a Pixmap of the specified size.
fn load_svg(contents: &[u8], size: u32) -> Result<Pixmap> {
    info!("Loading SVG content with size {}x{}", size, size);

    let options = Options::default();
    let fontdb = fontdb::Database::new();
    let tree: Tree = Tree::from_data(contents, &options, &fontdb).with_context(|| {
        format!(
            "Failed to create SVG tree from data of size {}x{}",
            size, size
        )
    })?;
    info!("Successfully created SVG tree");

    let mut pixmap: Pixmap = Pixmap::new(size, size).context("Failed to create a new Pixmap")?;
    info!("Created Pixmap of size {}x{}", size, size);

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
pub fn save_image(output: &PathBuf, format: &str, image: &str, size: u32) -> Result<()> {
    const SUPPORTED_FORMATS: &[&str] = &["svg", "png"];
    info!(
        "Starting to save image with format '{}' to {:?}",
        format, output
    );

    let file_path = output.with_extension(if SUPPORTED_FORMATS.contains(&format) {
        format
    } else {
        bail!("Unsupported image format: '{}'", format);
    });

    match format {
        "svg" => {
            let mut writer = BufWriter::new(
                File::create(&file_path)
                    .with_context(|| format!("Failed to create output file {:?}", file_path))?,
            );
            writer
                .write_all(image.as_bytes())
                .with_context(|| format!("Failed to write SVG image to file {:?}", file_path))?;
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
                .with_context(|| format!("Failed to save PNG image to file {:?}", file_path))?;
            info!("Saved PNG image to {:?}", file_path);
        }
        _ => {
            bail!("Unsupported image format: '{}'", format);
        }
    }

    info!("Image saved successfully to {:?}", file_path);
    Ok(())
}

/// An asynchronous wrapper for `save_image` for heavy I/O operations.
/// This function offloads blocking work to a separate thread using tokio's spawn_blocking.
pub async fn async_save_image(
    output: PathBuf,
    format: String,
    image: String,
    size: u32,
) -> Result<()> {
    tokio::task::spawn_blocking(move || save_image(&output, &format, &image, size))
        .await
        .expect("Task panicked")
}
