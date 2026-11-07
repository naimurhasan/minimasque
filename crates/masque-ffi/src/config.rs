#[derive(Debug, Clone, uniffi::Record)]
pub struct ForwarderConfig {
    pub proxy: String,
    pub server_name: String,
    pub ca_der: Vec<u8>,
    pub target: String,
    pub local: String,
    pub transport: String,
    pub template: Option<String>,
}
