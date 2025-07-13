// Error handling for EvalEds - GNU-style error formatting aligned with PromptEds
use thiserror::Error;

pub type Result<T> = std::result::Result<T, EvalError>;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl EvalError {
    /// Format error message in GNU-style format: program: command: error message
    pub fn format_error(&self, command: &str) -> String {
        match self {
            EvalError::NotFound(name) => {
                format!("evaleds: {}: evaluation '{}' not found", command, name)
            },
            EvalError::AlreadyExists(name) => {
                format!("evaleds: {}: evaluation '{}' already exists", command, name)
            },
            EvalError::ValidationError(msg) => {
                format!("evaleds: {}: {}", command, msg)
            },
            EvalError::ConfigError(msg) => {
                format!("evaleds: {}: configuration error: {}", command, msg)
            },
            EvalError::ProviderError(msg) => {
                format!("evaleds: {}: provider error: {}", command, msg)
            },
            EvalError::DatabaseError(msg) => {
                format!("evaleds: {}: database error: {}", command, msg)
            },
            EvalError::NetworkError(msg) => {
                format!("evaleds: {}: network error: {}", command, msg)
            },
            EvalError::IoError(err) => {
                format!("evaleds: {}: {}", command, err)
            },
            EvalError::PermissionDenied(msg) => {
                format!("evaleds: {}: permission denied: {}", command, msg)
            },
            EvalError::MissingDependency(dep) => {
                format!("evaleds: {}: missing dependency: {}", command, dep)
            },
            EvalError::SerializationError(msg) => {
                format!("evaleds: {}: serialization error: {}", command, msg)
            },
            EvalError::TemplateError(msg) => {
                format!("evaleds: {}: template error: {}", command, msg)
            },
            EvalError::AnalysisError(msg) => {
                format!("evaleds: {}: analysis error: {}", command, msg)
            },
        }
    }
    
    /// Get appropriate exit code for the error type
    pub fn exit_code(&self) -> i32 {
        match self {
            EvalError::NotFound(_) => 1,
            EvalError::AlreadyExists(_) => 1,
            EvalError::ValidationError(_) => 2,
            EvalError::ConfigError(_) => 3,
            EvalError::ProviderError(_) => 4,
            EvalError::DatabaseError(_) => 5,
            EvalError::NetworkError(_) => 6,
            EvalError::IoError(_) => 7,
            EvalError::PermissionDenied(_) => 8,
            EvalError::MissingDependency(_) => 9,
            EvalError::SerializationError(_) => 10,
            EvalError::TemplateError(_) => 11,
            EvalError::AnalysisError(_) => 12,
        }
    }
    
    /// Get helpful suggestions for common errors
    pub fn get_suggestion(&self) -> Option<String> {
        match self {
            EvalError::NotFound(_) => Some(
                "💡 Try running 'evaleds list' to see available evaluations\n💡 Or create a new one with 'evaleds create <name>'".to_string()
            ),
            EvalError::ProviderError(_) => Some(
                "💡 Check your API keys are set correctly:\n   - OPENAI_API_KEY for OpenAI\n   - ANTHROPIC_API_KEY for Anthropic\n   - GOOGLE_API_KEY for Google".to_string()
            ),
            EvalError::ConfigError(_) => Some(
                "💡 Check your configuration files:\n   - ~/.config/evaleds/config.toml\n   - ~/.evaleds/config.toml".to_string()
            ),
            EvalError::MissingDependency(_) => Some(
                "💡 Make sure all required dependencies are installed\n💡 Try running 'evaleds --version' to check installation".to_string()
            ),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for EvalError {
    fn from(err: sqlx::Error) -> Self {
        EvalError::DatabaseError(err.to_string())
    }
}

impl From<reqwest::Error> for EvalError {
    fn from(err: reqwest::Error) -> Self {
        EvalError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for EvalError {
    fn from(err: serde_json::Error) -> Self {
        EvalError::SerializationError(err.to_string())
    }
}

impl From<handlebars::RenderError> for EvalError {
    fn from(err: handlebars::RenderError) -> Self {
        EvalError::TemplateError(err.to_string())
    }
}

impl From<toml::de::Error> for EvalError {
    fn from(err: toml::de::Error) -> Self {
        EvalError::ConfigError(err.to_string())
    }
}

impl From<dialoguer::Error> for EvalError {
    fn from(err: dialoguer::Error) -> Self {
        EvalError::ValidationError(err.to_string())
    }
}