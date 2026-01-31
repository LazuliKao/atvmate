use std::fmt;

#[derive(Debug)]
pub enum ATVMateError {
    ADBError(String),
    DeviceError(String),
    ConfigurationError(String),
}

impl fmt::Display for ATVMateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ATVMateError::ADBError(msg) => write!(f, "ADB Error: {}", msg),
            ATVMateError::DeviceError(msg) => write!(f, "Device Error: {}", msg),
            ATVMateError::ConfigurationError(msg) => write!(f, "Configuration Error: {}", msg),
        }
    }
}

impl std::error::Error for ATVMateError {}
