use std::fmt;

use adb_client::RustADBError;
use poem::{
    IntoResponse, Response,
    http::{StatusCode, header::CONTENT_DISPOSITION},
};
use poem_openapi::{Object, payload::Json};

#[derive(Debug)]
pub enum ATVMateError {
    ADBError(String),
    DeviceError(String),
    ConfigurationError(String),
    FileNotFound(String),
    AppNotFound(String),
    ScreenshotError(String),
    FileTransferError(String),
}

#[derive(Object)]
struct ErrorResponse {
    success: bool,
    message: String,
}

pub struct BinaryResponse {
    pub data: Vec<u8>,
    pub content_type: String,
    pub filename: Option<String>,
}

impl fmt::Display for ATVMateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ATVMateError::ADBError(msg) => write!(f, "ADB Error: {}", msg),
            ATVMateError::DeviceError(msg) => write!(f, "Device Error: {}", msg),
            ATVMateError::ConfigurationError(msg) => {
                write!(f, "Configuration Error: {}", msg)
            }
            ATVMateError::FileNotFound(msg) => write!(f, "File Not Found: {}", msg),
            ATVMateError::AppNotFound(msg) => write!(f, "App Not Found: {}", msg),
            ATVMateError::ScreenshotError(msg) => write!(f, "Screenshot Error: {}", msg),
            ATVMateError::FileTransferError(msg) => write!(f, "File Transfer Error: {}", msg),
        }
    }
}

impl std::error::Error for ATVMateError {}

impl From<RustADBError> for ATVMateError {
    fn from(error: RustADBError) -> Self {
        Self::ADBError(error.to_string())
    }
}

impl IntoResponse for ATVMateError {
    fn into_response(self) -> Response {
        Json(ErrorResponse {
            success: false,
            message: self.to_string(),
        })
        .into_response()
    }
}

impl IntoResponse for BinaryResponse {
    fn into_response(self) -> Response {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .content_type(self.content_type);

        if let Some(filename) = self.filename {
            builder = builder.header(
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            );
        }

        builder.body(self.data)
    }
}
