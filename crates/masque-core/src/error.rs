use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnexpectedEnd,
    VarIntOverflow,
    InvalidTemplate,
    InvalidTarget,
    InvalidRequest,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => f.write_str("unexpected end of buffer"),
            Self::VarIntOverflow => f.write_str("value exceeds 2^62 - 1"),
            Self::InvalidTemplate => f.write_str("invalid uri template"),
            Self::InvalidTarget => f.write_str("invalid proxy target"),
            Self::InvalidRequest => f.write_str("not a connect-udp request"),
        }
    }
}

impl std::error::Error for Error {}
