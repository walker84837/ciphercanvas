use miette::Diagnostic;
use std::io;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("QR code generation error: {0}")]
    QrCode(String),
    #[error("Image processing error: {0}")]
    Image(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("File already exists: {0}")]
    FileExists(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
