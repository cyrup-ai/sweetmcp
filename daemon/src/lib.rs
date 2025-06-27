//! cyrupd - Cross-platform process supervision and daemon management
//!
//! This crate provides lightweight, production-ready daemon management
//! with crossbeam channels for wait-free message passing.

pub mod config;
pub mod daemon;
pub mod ipc;
pub mod lifecycle;
pub mod manager;
pub mod service;
pub mod state_machine;

// Re-export main types for convenience
pub use config::{ServiceConfig, ServiceDefinition, HealthCheckConfig, LogRotationConfig};
pub use daemon::daemonise;
pub use ipc::{Cmd, Evt};
pub use manager::ServiceManager;
pub use state_machine::{State, Event, Action, Transition};