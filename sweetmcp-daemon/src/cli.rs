use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "Cyrup service manager")]
pub struct Args {
    /// Sub‑commands (run, install, etc.)
    #[command(subcommand)]
    pub sub: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Normal daemon operation (default if no sub‑command)
    Run {
        /// Stay in foreground even on plain Unix
        #[arg(long)]
        foreground: bool,

        /// Path to configuration file
        #[arg(long, short = 'c')]
        config: Option<String>,

        /// Use system-wide config (/etc/cyrupd/cyrupd.toml)
        #[arg(long, conflicts_with = "config")]
        system: bool,
    },
    /// Copy binary, create users/dirs, register with init, start service
    Install {
        /// Don't enable & start the unit—copy files only
        #[arg(long)]
        dry_run: bool,

        /// macOS only – sign the binary after install (uses codesign)
        #[arg(long)]
        sign: bool,

        /// Override signing identity (default: ad‑hoc)
        #[arg(long)]
        identity: Option<String>,
    },
    /// Uninstall the daemon service
    Uninstall {
        /// Don't actually uninstall, just show what would be done
        #[arg(long)]
        dry_run: bool,
    },
}
