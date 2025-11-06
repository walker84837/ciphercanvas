use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("QR code generation error: {0}")]
    QrCode(String),
    #[error("Image processing error: {0}")]
    Image(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
