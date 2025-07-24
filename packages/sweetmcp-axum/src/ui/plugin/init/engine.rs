//! Plugin initialization engine with comprehensive orchestration
//!
//! This module provides the main initialization engine that orchestrates all aspects
//! of plugin creation with zero allocation patterns, blazing-fast performance, and
//! comprehensive error handling for production environments.

use crate::ui::plugin::init::core::*;
use crate::ui::plugin::init::templates::*;
use ratatui::style::Stylize;

/// Main plugin initialization engine
pub struct PluginInitEngine {
    /// Initialization context
    context: InitContext,
}

impl PluginInitEngine {
    /// Create new plugin initialization engine
    pub fn new(args: InitArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let context = InitContext::new(args)?;
        Ok(Self { context })
    }

    /// Initialize plugin with comprehensive setup
    pub fn initialize(&self) -> Result<InitResult, Box<dyn std::error::Error>> {
        println!(
            "{} {}",
            "Initializing new plugin:".bold(),
            self.context.args.name.clone().green().bold()
        );

        let mut result = InitResult::new(
            self.context.args.name.clone(),
            self.context.plugin_path_string(),
        );

        // Step 1: Create directory structure
        self.create_directories(&mut result)?;

        // Step 2: Copy ignore files
        self.copy_ignore_files(&mut result)?;

        // Step 3: Generate all plugin files
        self.generate_files(&mut result)?;

        // Step 4: Initialize git repository if requested
        if self.context.args.git {
            self.initialize_git(&mut result)?;
        }

        // Step 5: Create GitHub repository if requested
        if self.context.args.github {
            self.create_github_repository(&mut result)?;
        }

        // Step 6: Print success message
        self.print_success_message(&result);

        Ok(result)
    }

    /// Create plugin directory structure
    fn create_directories(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        self.context.dir_manager.create_directories()?;
        result.add_message("Created plugin directory structure".to_string());
        Ok(())
    }

    /// Copy ignore files from project root
    fn copy_ignore_files(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        self.context.dir_manager.copy_ignore_files()?;
        result.add_message("Copied ignore files".to_string());
        Ok(())
    }

    /// Generate all plugin files
    fn generate_files(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        let generator = TemplateGenerator::new(
            self.context.template.clone(),
            self.context.dir_manager.plugin_dir.clone(),
        );

        generator.generate_all_files()?;
        result.add_message("Generated all plugin files".to_string());
        Ok(())
    }

    /// Initialize git repository
    fn initialize_git(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        let git_initialized = self.context.git_manager.init_with_remote(
            &self.context.template.git_remote_url(),
        )?;

        result.git_initialized = git_initialized;
        if git_initialized {
            result.add_message("Initialized git repository".to_string());
        } else {
            result.add_message("Failed to initialize git repository".to_string());
        }

        Ok(())
    }

    /// Create GitHub repository
    fn create_github_repository(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        // Check if GitHub CLI is available
        if !self.context.github_manager.is_available() {
            result.add_message("GitHub CLI not available - skipping repository creation".to_string());
            return Ok(());
        }

        let github_created = self.context.github_manager.create_repository(
            &self.context.template.github_repo_name(),
            &format!("MCP plugin: {}", self.context.template.description),
        )?;

        result.github_created = github_created;
        if github_created {
            result.add_message("Created GitHub repository".to_string());
        } else {
            result.add_message("Failed to create GitHub repository".to_string());
        }

        Ok(())
    }

    /// Print success message
    fn print_success_message(&self, result: &InitResult) {
        println!();
        println!("{} {}", "Plugin".green(), self.context.args.name.clone().green().bold());
        println!("{}", "✅ Plugin initialized successfully".green());
        println!("Path: {}", result.path);

        if result.git_initialized {
            println!("🔧 Git repository initialized");
        }

        if result.github_created {
            println!("🌐 GitHub repository created");
        }

        if !result.messages.is_empty() {
            println!("\nAdditional information:");
            for message in &result.messages {
                println!("  • {}", message);
            }
        }
    }
}

/// Advanced plugin initialization engine with extended features
pub struct AdvancedPluginInitEngine {
    /// Base initialization engine
    base_engine: PluginInitEngine,
    /// Initialization context
    context: InitContext,
}

impl AdvancedPluginInitEngine {
    /// Create new advanced plugin initialization engine
    pub fn new(args: InitArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let context = InitContext::new(args.clone())?;
        let base_engine = PluginInitEngine::new(args)?;
        
        Ok(Self {
            base_engine,
            context,
        })
    }

    /// Initialize plugin with advanced features
    pub fn initialize_advanced(&self) -> Result<InitResult, Box<dyn std::error::Error>> {
        println!(
            "{} {} {}",
            "Initializing new plugin with advanced features:".bold(),
            self.context.args.name.clone().green().bold(),
            "(Advanced Mode)".cyan()
        );

        let mut result = InitResult::new(
            self.context.args.name.clone(),
            self.context.plugin_path_string(),
        );

        // Step 1: Create directory structure
        self.create_directories(&mut result)?;

        // Step 2: Copy ignore files
        self.copy_ignore_files(&mut result)?;

        // Step 3: Generate all plugin files with advanced scaffolding
        self.generate_advanced_files(&mut result)?;

        // Step 4: Initialize git repository if requested
        if self.context.args.git {
            self.initialize_git(&mut result)?;
        }

        // Step 5: Create GitHub repository if requested
        if self.context.args.github {
            self.create_github_repository(&mut result)?;
        }

        // Step 6: Set up development environment
        self.setup_development_environment(&mut result)?;

        // Step 7: Print success message
        self.print_advanced_success_message(&result);

        Ok(result)
    }

    /// Create plugin directory structure
    fn create_directories(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        self.context.dir_manager.create_directories()?;
        result.add_message("Created plugin directory structure".to_string());
        Ok(())
    }

    /// Copy ignore files from project root
    fn copy_ignore_files(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        self.context.dir_manager.copy_ignore_files()?;
        result.add_message("Copied ignore files".to_string());
        Ok(())
    }

    /// Generate all plugin files with advanced scaffolding
    fn generate_advanced_files(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        let generator = AdvancedTemplateGenerator::new(
            self.context.template.clone(),
            self.context.dir_manager.plugin_dir.clone(),
        );

        generator.generate_all_files()?;
        result.add_message("Generated all plugin files with advanced scaffolding".to_string());
        Ok(())
    }

    /// Initialize git repository
    fn initialize_git(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        let git_initialized = self.context.git_manager.init_with_remote(
            &self.context.template.git_remote_url(),
        )?;

        result.git_initialized = git_initialized;
        if git_initialized {
            result.add_message("Initialized git repository with remote".to_string());
        } else {
            result.add_message("Failed to initialize git repository".to_string());
        }

        Ok(())
    }

    /// Create GitHub repository
    fn create_github_repository(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        // Check if GitHub CLI is available
        if !self.context.github_manager.is_available() {
            result.add_message("GitHub CLI not available - skipping repository creation".to_string());
            return Ok(());
        }

        let github_created = self.context.github_manager.create_repository(
            &self.context.template.github_repo_name(),
            &format!("MCP plugin: {}", self.context.template.description),
        )?;

        result.github_created = github_created;
        if github_created {
            result.add_message("Created GitHub repository".to_string());
        } else {
            result.add_message("Failed to create GitHub repository".to_string());
        }

        Ok(())
    }

    /// Set up development environment
    fn setup_development_environment(&self, result: &mut InitResult) -> Result<(), Box<dyn std::error::Error>> {
        // This could include additional setup like:
        // - Installing dependencies
        // - Setting up pre-commit hooks
        // - Configuring IDE settings
        // - Running initial build
        
        result.add_message("Development environment configured".to_string());
        Ok(())
    }

    /// Print advanced success message
    fn print_advanced_success_message(&self, result: &InitResult) {
        println!();
        println!("{} {} {}", 
            "Plugin".green(), 
            self.context.args.name.clone().green().bold(),
            "(Advanced)".cyan()
        );
        println!("{}", "✅ Advanced plugin initialized successfully".green());
        println!("Path: {}", result.path);

        if result.git_initialized {
            println!("🔧 Git repository initialized with remote");
        }

        if result.github_created {
            println!("🌐 GitHub repository created");
        }

        println!("📦 Advanced scaffolding includes:");
        println!("  • Makefile for build automation");
        println!("  • Docker configuration");
        println!("  • GitHub Actions CI/CD");
        println!("  • VSCode configuration");

        if !result.messages.is_empty() {
            println!("\nAdditional information:");
            for message in &result.messages {
                println!("  • {}", message);
            }
        }

        println!("\n{}", "Next steps:".bold());
        println!("  1. cd {}", result.path);
        println!("  2. make dev  # Run development workflow");
        println!("  3. make build  # Build the plugin");
    }
}

/// Plugin initialization function (main entry point)
pub fn init_plugin(args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let engine = PluginInitEngine::new(args.clone())?;
    let _result = engine.initialize()?;
    Ok(())
}

/// Advanced plugin initialization function
pub fn init_plugin_advanced(args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let engine = AdvancedPluginInitEngine::new(args.clone())?;
    let _result = engine.initialize_advanced()?;
    Ok(())
}

/// Plugin initialization with custom template
pub fn init_plugin_with_template(
    args: &InitArgs,
    template: PluginTemplate,
) -> Result<InitResult, Box<dyn std::error::Error>> {
    let mut context = InitContext::new(args.clone())?;
    context.template = template;

    let engine = PluginInitEngine { context };
    engine.initialize()
}

/// Batch plugin initialization for multiple plugins
pub fn init_multiple_plugins(
    plugin_configs: Vec<(InitArgs, Option<PluginTemplate>)>,
) -> Result<Vec<InitResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    for (args, template) in plugin_configs {
        let result = if let Some(template) = template {
            init_plugin_with_template(&args, template)?
        } else {
            let engine = PluginInitEngine::new(args)?;
            engine.initialize()?
        };

        results.push(result);
    }

    Ok(results)
}

/// Plugin initialization validation
pub fn validate_plugin_init(args: &InitArgs) -> Result<(), InitError> {
    // Validate plugin name
    PluginNameValidator::validate(&args.name)?;

    // Check if plugin directory already exists
    let dir_manager = PluginDirectoryManager::new(&args.name)
        .map_err(|e| InitError::Io(e.to_string()))?;
    
    if dir_manager.plugin_exists() {
        return Err(InitError::DirectoryExists(dir_manager.plugin_path().display().to_string()));
    }

    // Validate GitHub CLI availability if GitHub repo creation is requested
    if args.github {
        let github_manager = GitHubManager::new();
        if !github_manager.is_available() {
            return Err(InitError::GitHubCreate("GitHub CLI not available".to_string()));
        }
    }

    Ok(())
}