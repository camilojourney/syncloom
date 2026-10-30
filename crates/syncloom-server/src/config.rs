//! Where to bind. The server listens on exactly `127.0.0.1`; the port is free.
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const DEFAULT_ADDRESS: &str = "127.0.0.1:8790";

pub fn local_address(value: Option<&str>) -> Result<SocketAddr, &'static str> {
    let address: SocketAddr = value
        .unwrap_or(DEFAULT_ADDRESS)
        .parse()
        .map_err(|_| "SYNCLOOM_ADDR must be 127.0.0.1:<port>")?;
    if address.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST) {
        return Err("SYNCLOOM_ADDR must use 127.0.0.1");
    }
    Ok(address)
}
