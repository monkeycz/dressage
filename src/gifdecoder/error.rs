use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IoError")]
    IoError(#[from] std::io::Error),
    #[error("Utf8Error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("FormatError")]
    FormatError,
    #[error("InvalidVersion")]
    InvalidVersion,
    #[error("InvalidExtension")]
    InvalidExtension,
    #[error("BlockNotTerminated")]
    BlockNotTerminated,
    #[error("LzwError")]
    LzwError(#[from] weezl::LzwError),
}

pub type Result<T> = std::result::Result<T, Error>;

