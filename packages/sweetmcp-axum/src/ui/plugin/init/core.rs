//! Core plugin initialization structures and types
//!
//! This module provides the core functionality for initializing new MCP plugins
//! with zero allocation patterns, blazing-fast performance, and comprehensive
//! project scaffolding for production environments.

use std::{fs, process::Command};
use clap::Args;
use ratatui::style::Stylize;

/// Plugin initialization arguments
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Name of the plugin to create
    #[arg(short, long)]
    pub name: String,

    /// Initialize git repository for the plugin
    #[arg(short, long)]
    pub git: bool,

    /// Create GitHub repository for the plugin
    #[arg(short, long)]
    pub github: bool,

    /// Plugin description
    #[arg(short, long, default_value = "MCP plugin")]
    pub description: String,
}

/// Plugin initialization result
#[derive(Debug, Clone)]
pub struct InitResult {
    /// Plugin name
    pub name: String,
    /// Plugin directory path
    pub path: String,
    /// Whether git was initialized
    pub git_initialized: bool,
    /// Whether GitHub repository was created
    pub github_created: bool,
    /// Any warnings or messages
    pub messages: Vec<String>,
}

impl InitResult {
    /// Create new initialization result
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            git_initialized: false,
            github_created: false,
            messages: Vec::new(),
        }
    }

    /// Add message to result
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    /// Check if initialization was successful
    pub fn is_successful(&self) -> bool {
        !self.path.is_empty()
    }
}

/// Plugin template configuration
#[derive(Debug, Clone)]
pub struct PluginTemplate {
    /// Plugin name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Author information
    pub author: String,
    /// License type
    pub license: String,
    /// Repository URL template
    pub repository_template: String,
}

impl PluginTemplate {
    /// Create new plugin template with defaults
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            author: "CYRUP.ai Dev Team".to_string(),
            license: "Apache-2.0".to_string(),
            repository_template: "https://github.com/cyrup-ai/mcp-plugin-{}.git".to_string(),
        }
    }

    /// Get repository URL for this plugin
    pub fn repository_url(&self) -> String {
        self.repository_template.replace("{}", &self.name)
    }

    /// Get package name for this plugin
    pub fn package_name(&self) -> String {
        format!("mcp-plugin-{}", self.name)
    }

    /// Get GitHub repository name
    pub fn github_repo_name(&self) -> String {
        format!("cyrup-ai/mcp-plugin-{}", self.name)
    }

    /// Get git remote URL
    pub fn git_remote_url(&self) -> String {
        format!("git@github.com:cyrup-ai/mcp-plugin-{}.git", self.name)
    }
}

/// Plugin directory structure manager
#[derive(Debug)]
pub struct PluginDirectoryManager {
    /// Base plugins directory
    pub plugins_dir: std::path::PathBuf,
    /// Current plugin directory
    pub plugin_dir: std::path::PathBuf,
}

impl PluginDirectoryManager {
    /// Create new directory manager
    pub fn new(plugin_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let project_root = std::env::current_dir()?;
        let plugins_dir = project_root.join("plugins");
        let plugin_dir = plugins_dir.join(plugin_name);

        Ok(Self {
            plugins_dir,
            plugin_dir,
        })
    }

    /// Check if plugin directory already exists
    pub fn plugin_exists(&self) -> bool {
        self.plugin_dir.exists()
    }

    /// Create plugin directory structure
    pub fn create_directories(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create plugins directory if it doesn't exist
        if !self.plugins_dir.exists() {
            fs::create_dir_all(&self.plugins_dir)?;
            println!("Created plugins directory at {}", self.plugins_dir.display());
        }

        // Check if plugin directory already exists
        if self.plugin_dir.exists() {
            return Err(format!("Plugin directory already exists: {}", self.plugin_dir.display()).into());
        }

        // Create plugin directory structure
        fs::create_dir_all(&self.plugin_dir)?;
        fs::create_dir_all(self.plugin_dir.join("src"))?;
        fs::create_dir_all(self.plugin_dir.join("src").join("plugin"))?;

        Ok(())
    }

    /// Copy ignore files from project root
    pub fn copy_ignore_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let project_root = std::env::current_dir()?;
        
        for ignore_file in [".gitignore", ".cursorignore", ".aiderignore"] {
            let root_ignore = project_root.join(ignore_file);
            if root_ignore.exists() {
                fs::copy(&root_ignore, self.plugin_dir.join(ignore_file))?;
                println!("Copied {} to plugin directory", ignore_file);
            }
        }

        Ok(())
    }

    /// Get plugin directory path
    pub fn plugin_path(&self) -> &std::path::Path {
        &self.plugin_dir
    }

    /// Get plugins directory path
    pub fn plugins_path(&self) -> &std::path::Path {
        &self.plugins_dir
    }
}

/// Git repository manager for plugin initialization
#[derive(Debug)]
pub struct GitManager {
    /// Plugin directory
    plugin_dir: std::path::PathBuf,
}

impl GitManager {
    /// Create new git manager
    pub fn new(plugin_dir: std::path::PathBuf) -> Self {
        Self { plugin_dir }
    }

    /// Initialize git repository
    pub fn init_repository(&self) -> Result<bool, Box<dyn std::error::Error>> {
        println!("Initializing git repository...");

        let git_init = Command::new("git")
            .args(["init"])
            .current_dir(&self.plugin_dir)
            .output()?;

        if git_init.status.success() {
            println!("Git repository initialized");
            Ok(true)
        } else {
            println!(
                "Failed to initialize git repository: {}",
                String::from_utf8_lossy(&git_init.stderr)
            );
            Ok(false)
        }
    }

    /// Set up git remote
    pub fn setup_remote(&self, remote_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let git_remote = Command::new("git")
            .args(["remote", "add", "origin", remote_url])
            .current_dir(&self.plugin_dir)
            .output()?;

        if git_remote.status.success() {
            println!("Git remote set to {}", remote_url);
            Ok(true)
        } else {
            println!(
                "Failed to set git remote: {}",
                String::from_utf8_lossy(&git_remote.stderr)
            );
            Ok(false)
        }
    }

    /// Initialize git repository with remote
    pub fn init_with_remote(&self, remote_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let git_initialized = self.init_repository()?;
        if git_initialized {
            self.setup_remote(remote_url)
        } else {
            Ok(false)
        }
    }
}

/// GitHub repository manager for plugin initialization
#[derive(Debug)]
pub struct GitHubManager;

impl GitHubManager {
    /// Create new GitHub manager
    pub fn new() -> Self {
        Self
    }

    /// Create GitHub repository
    pub fn create_repository(
        &self,
        repo_name: &str,
        description: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        println!("Creating GitHub repository...");

        let gh_create = Command::new("gh")
            .args([
                "repo",
                "create",
                repo_name,
                "--description",
                description,
                "--private",
            ])
            .output()?;

        if gh_create.status.success() {
            println!("GitHub repository created: {}", repo_name);
            Ok(true)
        } else {
            println!(
                "Failed to create GitHub repository: {}",
                String::from_utf8_lossy(&gh_create.stderr)
            );
            Ok(false)
        }
    }

    /// Check if GitHub CLI is available
    pub fn is_available(&self) -> bool {
        Command::new("gh")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl Default for GitHubManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin initialization error types
#[derive(Debug, Clone)]
pub enum InitError {
    /// Plugin directory already exists
    DirectoryExists(String),
    /// Failed to create directories
    DirectoryCreation(String),
    /// Failed to write files
    FileWrite(String),
    /// Git initialization failed
    GitInit(String),
    /// GitHub repository creation failed
    GitHubCreate(String),
    /// Invalid plugin name
    InvalidName(String),
    /// IO error
    Io(String),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::DirectoryExists(path) => write!(f, "Plugin directory already exists: {}", path),
            InitError::DirectoryCreation(msg) => write!(f, "Failed to create directories: {}", msg),
            InitError::FileWrite(msg) => write!(f, "Failed to write files: {}", msg),
            InitError::GitInit(msg) => write!(f, "Git initialization failed: {}", msg),
            InitError::GitHubCreate(msg) => write!(f, "GitHub repository creation failed: {}", msg),
            InitError::InvalidName(name) => write!(f, "Invalid plugin name: {}", name),
            InitError::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for InitError {}

/// Plugin name validator
pub struct PluginNameValidator;

impl PluginNameValidator {
    /// Validate plugin name
    pub fn validate(name: &str) -> Result<(), InitError> {
        if name.is_empty() {
            return Err(InitError::InvalidName("Plugin name cannot be empty".to_string()));
        }

        if name.len() > 50 {
            return Err(InitError::InvalidName("Plugin name too long (max 50 characters)".to_string()));
        }

        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(InitError::InvalidName(
                "Plugin name can only contain alphanumeric characters, hyphens, and underscores".to_string(),
            ));
        }

        // Check for reserved names
        let reserved_names = ["test", "example", "demo", "template"];
        if reserved_names.contains(&name.to_lowercase().as_str()) {
            return Err(InitError::InvalidName(format!("'{}' is a reserved name", name)));
        }

        Ok(())
    }

    /// Normalize plugin name
    pub fn normalize(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}

/// Plugin initialization context
#[derive(Debug)]
pub struct InitContext {
    /// Initialization arguments
    pub args: InitArgs,
    /// Plugin template
    pub template: PluginTemplate,
    /// Directory manager
    pub dir_manager: PluginDirectoryManager,
    /// Git manager
    pub git_manager: GitManager,
    /// GitHub manager
    pub github_manager: GitHubManager,
}

impl InitContext {
    /// Create new initialization context
    pub fn new(args: InitArgs) -> Result<Self, Box<dyn std::error::Error>> {
        // Validate plugin name
        PluginNameValidator::validate(&args.name)?;

        let template = PluginTemplate::new(args.name.clone(), args.description.clone());
        let dir_manager = PluginDirectoryManager::new(&args.name)?;
        let git_manager = GitManager::new(dir_manager.plugin_dir.clone());
        let github_manager = GitHubManager::new();

        Ok(Self {
            args,
            template,
            dir_manager,
            git_manager,
            github_manager,
        })
    }

    /// Get plugin directory path as string
    pub fn plugin_path_string(&self) -> String {
        self.dir_manager.plugin_dir.display().to_string()
    }
}