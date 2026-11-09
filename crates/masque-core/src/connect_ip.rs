use crate::{Error, percent, uri_template};

pub const PROTOCOL: &str = "connect-ip";

pub const WELL_KNOWN_PATH: &str = "/.well-known/masque/ip/{target}/{ipproto}/";

pub const IP_PACKET: u64 = 0x00;

const TARGET: &str = "target";
const IPPROTO: &str = "ipproto";
const WILDCARD: &str = "*";

pub fn well_known_url(authority: &str) -> String {
    format!("https://{authority}{WELL_KNOWN_PATH}")
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    pub target: Option<String>,
    pub ipproto: Option<u8>,
}

fn encode(value: &str) -> String {
    match value {
        WILDCARD => WILDCARD.into(),
        other => percent::encode(other),
    }
}

pub fn expand(template: &str, scope: &Scope) -> Result<String, Error> {
    let target = scope.target.as_deref().unwrap_or(WILDCARD);
    let ipproto = scope
        .ipproto
        .map_or_else(|| WILDCARD.into(), |proto| proto.to_string());
    uri_template::expand_with(template, &[(TARGET, target), (IPPROTO, &ipproto)], encode)
}

pub fn accept(
    method: &str,
    protocol: Option<&str>,
    path: &str,
    template: &str,
) -> Result<Scope, Error> {
    if method != "CONNECT" || protocol != Some(PROTOCOL) {
        return Err(Error::InvalidRequest);
    }
    parse(template, path)
}

fn wildcard_or<T>(value: String, parse: impl Fn(String) -> Option<T>) -> Result<Option<T>, Error> {
    match value.as_str() {
        WILDCARD => Ok(None),
        "" => Err(Error::InvalidTarget),
        _ => parse(value).map(Some).ok_or(Error::InvalidTarget),
    }
}

pub fn parse(template: &str, path: &str) -> Result<Scope, Error> {
    let found = uri_template::captures(template, path).ok_or(Error::InvalidTarget)?;
    let value = |name: &str| {
        found
            .iter()
            .find(|(var, _)| *var == name)
            .and_then(|(_, raw)| percent::decode(raw))
            .ok_or(Error::InvalidTarget)
    };
    Ok(Scope {
        target: wildcard_or(value(TARGET)?, Some)?,
        ipproto: wildcard_or(value(IPPROTO)?, |raw| raw.parse().ok())?,
    })
}
