use std::fmt;

use masque_core::Target;
use masque_core::ip_capsule::Prefix;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remote {
    Udp(Target),
    Ip(Prefix),
}

impl Remote {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Udp(_) => "udp",
            Self::Ip(_) => "ip",
        }
    }
}

impl From<Target> for Remote {
    fn from(target: Target) -> Self {
        Self::Udp(target)
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Udp(target) => write!(f, "{}:{}", target.host, target.port),
            Self::Ip(prefix) => write!(f, "{}/{}", prefix.addr, prefix.len),
        }
    }
}
