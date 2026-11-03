//! syncloom-server: the loopback HTTP API of the studio (`studio-api`).
mod boundary;
mod capture;
mod config;
mod connection_routes;
mod failure;
mod meta;
mod page;
mod request_scope;
mod server;
mod social_handlers;
mod social_routes;

pub use config::{DEFAULT_ADDRESS, local_address};
pub use request_scope::log_subscriber;
pub use server::serve;
