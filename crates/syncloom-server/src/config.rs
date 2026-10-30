//! Where to bind. The server listens on exactly `127.0.0.1`; the port is free.
use std::net::SocketAddr;

pub const DEFAULT_ADDRESS: &str = "127.0.0.1:8790";

pub fn local_address(value: Option<&str>) -> Result<SocketAddr, &'static str> {
    let _ = value;
    Err("SYNCLOOM_ADDR must be 127.0.0.1:<port>")
}
