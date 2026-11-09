use std::net::IpAddr;

use masque_core::ip_capsule::Prefix;
use masque_h3::Error;
use tun_rs::{AsyncDevice, DeviceBuilder};

pub const MTU: u16 = 1150;

pub fn open(address: Prefix) -> Result<AsyncDevice, Error> {
    let IpAddr::V4(v4) = address.addr else {
        return Err("only IPv4 tunnel addresses are supported".into());
    };
    Ok(DeviceBuilder::new()
        .mtu(MTU)
        .ipv4(v4, address.len, None)
        .build_async()?)
}

pub fn name(device: &AsyncDevice) -> Result<String, Error> {
    Ok(device.name()?)
}
