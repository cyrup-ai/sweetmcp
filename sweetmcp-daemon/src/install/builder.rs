use std::{collections::HashMap, path::PathBuf};

/// Builder for daemon installation metadata.
///
/// This struct describes the daemon to be installed, including its executable path,
/// arguments, environment variables, and service configuration.
#[derive(Debug, Clone)]
pub struct InstallerBuilder {
    /// Service identifier (systemd unit name, launchd label, Windows service name)
    pub label: String,

    /// Path to the daemon executable
    pub program: PathBuf,

    /// Command line arguments for the daemon
    pub args: Vec<String>,

    /// Environment variables to set for the daemon process
    pub env: HashMap<String, String>,

    /// User account to run the daemon as
    pub run_as_user: String,

    /// Group to run the daemon as (Unix only)
    pub run_as_group: String,

    /// Human-readable description of the service
    pub description: String,

    /// Whether to automatically restart on failure
    pub auto_restart: bool,

    /// Whether the daemon requires network availability
    pub wants_network: bool,
}

impl InstallerBuilder {
    /// Create a new installer configuration.
    ///
    /// # Arguments
    ///
    /// * `label` - Unique identifier for the service (e.g., "my-daemon")
    /// * `program` - Path to the daemon executable
    pub fn new(label: &str, program: impl Into<PathBuf>) -> Self {
        Self {
            label: label.to_string(),
            program: program.into(),
            args: Vec::new(),
            env: HashMap::new(),
            run_as_user: "daemon".into(),
            run_as_group: "daemon".into(),
            description: format!("{} service", label),
            auto_restart: true,
            wants_network: true,
        }
    }

    /// Add a command line argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple command line arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set an environment variable.
    pub fn env(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.env.insert(k.into(), v.into());
        self
    }

    /// Set the user account to run as.
    pub fn user(mut self, u: impl Into<String>) -> Self {
        self.run_as_user = u.into();
        self
    }

    /// Set the group to run as (Unix only).
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.run_as_group = g.into();
        self
    }

    /// Set the service description.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    /// Enable or disable automatic restart on failure.
    pub fn auto_restart(mut self, v: bool) -> Self {
        self.auto_restart = v;
        self
    }

    /// Specify whether the daemon requires network availability.
    pub fn network(mut self, v: bool) -> Self {
        self.wants_network = v;
        self
    }
}

/// Builder for privileged command execution.
///
/// This is used internally for constructing platform-specific installation commands.
#[derive(Debug)]
pub struct CommandBuilder {
    /// Program to execute
    pub program: PathBuf,

    /// Arguments for the program
    pub args: Vec<String>,
}

impl CommandBuilder {
    /// Create a new command builder.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    /// Add a command line argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple command line arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}
