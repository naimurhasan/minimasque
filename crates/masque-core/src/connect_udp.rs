use std::str::FromStr;

use crate::{Error, percent, uri_template};

pub const PROTOCOL: &str = "connect-udp";

pub const WELL_KNOWN_PATH: &str = "/.well-known/masque/udp/{target_host}/{target_port}/";

const HOST: &str = "target_host";
const PORT: &str = "target_port";

pub fn well_known_url(authority: &str) -> String {
    format!("https://{authority}{WELL_KNOWN_PATH}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub host: String,
    pub port: u16,
}

pub fn expand(template: &str, target: &Target) -> Result<String, Error> {
    uri_template::expand(
        template,
        &[(HOST, &target.host), (PORT, &target.port.to_string())],
    )
}

pub fn accept(
    method: &str,
    protocol: Option<&str>,
    path: &str,
    template: &str,
) -> Result<Target, Error> {
    if method != "CONNECT" || protocol != Some(PROTOCOL) {
        return Err(Error::InvalidRequest);
    }
    parse(template, path)
}

pub fn parse(template: &str, path: &str) -> Result<Target, Error> {
    let found = uri_template::captures(template, path).ok_or(Error::InvalidTarget)?;
    let value = |name: &str| {
        found
            .iter()
            .find(|(var, _)| *var == name)
            .and_then(|(_, raw)| percent::decode(raw))
            .filter(|decoded| !decoded.is_empty())
            .ok_or(Error::InvalidTarget)
    };
    let host = value(HOST)?;
    let port = value(PORT)?.parse().map_err(|_| Error::InvalidTarget)?;
    Ok(Target { host, port })
}

impl FromStr for Target {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Error> {
        let (host, port) = input.rsplit_once(':').ok_or(Error::InvalidTarget)?;
        let host = match host.strip_prefix('[') {
            Some(bracketed) => bracketed.strip_suffix(']').ok_or(Error::InvalidTarget)?,
            None if host.contains(':') => return Err(Error::InvalidTarget),
            None => host,
        };
        let port = port.parse().map_err(|_| Error::InvalidTarget)?;
        if host.is_empty() {
            return Err(Error::InvalidTarget);
        }
        Ok(Self {
            host: host.into(),
            port,
        })
    }
}
