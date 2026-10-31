//! syncloom-server: the loopback HTTP API of the studio (`studio-api`).
mod boundary;
mod config;
mod failure;
mod page;
mod request_scope;
mod server;

pub use config::{DEFAULT_ADDRESS, local_address};
pub use request_scope::log_subscriber;
pub use server::serve;
