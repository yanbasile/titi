//! Redititi - Redis-like server for terminal automation
//!
//! This module implements the Redititi server that enables
//! programmatic control of terminal sessions and panes.

pub mod auth;
pub mod channels;
pub mod commands;
pub mod protocol;
pub mod registry;
pub mod redititi_tcp_server;

pub use auth::{TokenAuth, AuthError};
pub use channels::{ChannelManager, Message};
pub use commands::CommandHandler;
pub use protocol::{Protocol, Response};
pub use registry::{Registry, SessionInfo, PaneInfo};
pub use redititi_tcp_server::RedititiTcpServer;
