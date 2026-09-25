use derive_more::{Display, From};

/// `ttd` crate specifc result type
pub type Result<T> = std::result::Result<T, Error>;

/// `ttd` crate specifc error type
#[derive(From, Debug, Display, Default)]
pub enum Error {
    #[default]
    Unknown,

    Generic(String),

    MissingValue,
    NotFound,
    DataMismatch,
    InitializationError,
    ConversionError,
    ForeignFunctionError,

    #[from]
    TtdSysError(ttd_sys::error::Error),

    #[from]
    Utf8Error(std::str::Utf8Error),

    #[from]
    Utf16Error(std::string::FromUtf16Error),

    #[from]
    Null(std::ffi::NulError),

    #[from]
    ParseError(std::num::ParseIntError),

    #[from]
    LoggerError(log::SetLoggerError),
}

impl std::error::Error for Error {}
