// EvalEds library - AI evaluation platform with PromptEds integration

pub mod cli;
pub mod core;
pub mod utils;
pub mod web;

// Re-export commonly used types
pub use core::evaluation::*;
pub use utils::error::{Result, EvalError};

/// EvalEds version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Initialize EvalEds with default configuration
pub async fn init() -> Result<()> {
    // Create necessary directories
    let home_dir = dirs::home_dir()
        .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?;
    
    let evaleds_dir = home_dir.join(".evaleds");
    tokio::fs::create_dir_all(&evaleds_dir).await
        .map_err(|e| EvalError::IoError(e))?;
    
    let config_dir = evaleds_dir.join("config");
    tokio::fs::create_dir_all(&config_dir).await
        .map_err(|e| EvalError::IoError(e))?;
    
    println!("✅ EvalEds initialized successfully");
    println!("📁 Configuration directory: {}", evaleds_dir.display());
    
    Ok(())
}

/// Get EvalEds configuration directory
pub fn get_config_dir() -> Result<std::path::PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?;
    
    Ok(home_dir.join(".evaleds"))
}

/// Get EvalEds version information
pub fn version_info() -> String {
    format!("{} v{} - {}", NAME, VERSION, DESCRIPTION)
}