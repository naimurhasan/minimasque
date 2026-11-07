use std::fmt;

#[derive(Debug, uniffi::Error)]
#[uniffi(flat_error)]
pub enum MasqueError {
    Failed(String),
}

impl fmt::Display for MasqueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for MasqueError {}

pub fn failed(err: impl fmt::Display) -> MasqueError {
    MasqueError::Failed(err.to_string())
}
