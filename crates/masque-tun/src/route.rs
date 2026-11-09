use masque_core::ip_capsule::Prefix;
use masque_h3::Error;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    MacOs,
}

#[cfg(target_os = "macos")]
pub const CURRENT: Os = Os::MacOs;
#[cfg(not(target_os = "macos"))]
pub const CURRENT: Os = Os::Linux;

pub fn command(os: Os, prefix: &Prefix, device: &str) -> Vec<String> {
    let prefix = prefix.to_string();
    let parts: Vec<&str> = match os {
        Os::Linux => vec!["ip", "route", "replace", &prefix, "dev", device],
        Os::MacOs => vec!["route", "-n", "add", "-net", &prefix, "-interface", device],
    };
    parts.into_iter().map(str::to_owned).collect()
}

pub async fn add(prefix: &Prefix, device: &str) -> Result<(), Error> {
    let argv = command(CURRENT, prefix, device);
    let status = Command::new(&argv[0]).args(&argv[1..]).status().await?;
    if !status.success() {
        return Err(format!("`{}` failed with {status}", argv.join(" ")).into());
    }
    Ok(())
}
