// Configuration management for EvalEds - PromptEds aligned patterns
use crate::utils::error::{Result, EvalError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalEdsConfig {
    pub providers: HashMap<String, ProviderInfo>,
    pub defaults: DefaultSettings,
    pub analysis: AnalysisSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub available_models: Vec<String>,
    pub default_model: String,
    pub pricing: HashMap<String, ModelPricing>,
    pub rate_limits: RateLimits,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSettings {
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub max_concurrent: u32,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSettings {
    pub enable_similarity_analysis: bool,
    pub enable_content_analysis: bool,
    pub enable_quality_assessment: bool,
    pub similarity_threshold: f32,
    pub max_keywords: usize,
}

impl Default for EvalEdsConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // OpenAI configuration
        providers.insert("openai".to_string(), ProviderInfo {
            available_models: vec![
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            default_model: "gpt-4-turbo".to_string(),
            pricing: {
                let mut pricing = HashMap::new();
                pricing.insert("gpt-4".to_string(), ModelPricing {
                    input_price_per_1k: 0.03,
                    output_price_per_1k: 0.06,
                });
                pricing.insert("gpt-4-turbo".to_string(), ModelPricing {
                    input_price_per_1k: 0.01,
                    output_price_per_1k: 0.03,
                });
                pricing.insert("gpt-3.5-turbo".to_string(), ModelPricing {
                    input_price_per_1k: 0.001,
                    output_price_per_1k: 0.002,
                });
                pricing
            },
            rate_limits: RateLimits {
                requests_per_minute: 3500,
                tokens_per_minute: 90000,
            },
            enabled: true,
        });
        
        // Anthropic configuration
        providers.insert("anthropic".to_string(), ProviderInfo {
            available_models: vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
            default_model: "claude-3-sonnet-20240229".to_string(),
            pricing: {
                let mut pricing = HashMap::new();
                pricing.insert("claude-3-opus-20240229".to_string(), ModelPricing {
                    input_price_per_1k: 0.015,
                    output_price_per_1k: 0.075,
                });
                pricing.insert("claude-3-sonnet-20240229".to_string(), ModelPricing {
                    input_price_per_1k: 0.003,
                    output_price_per_1k: 0.015,
                });
                pricing.insert("claude-3-haiku-20240307".to_string(), ModelPricing {
                    input_price_per_1k: 0.00025,
                    output_price_per_1k: 0.00125,
                });
                pricing
            },
            rate_limits: RateLimits {
                requests_per_minute: 1000,
                tokens_per_minute: 100000,
            },
            enabled: true,
        });
        
        // Google configuration
        providers.insert("google".to_string(), ProviderInfo {
            available_models: vec![
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
                "gemini-pro".to_string(),
            ],
            default_model: "gemini-1.5-pro".to_string(),
            pricing: {
                let mut pricing = HashMap::new();
                pricing.insert("gemini-1.5-pro".to_string(), ModelPricing {
                    input_price_per_1k: 0.0035,
                    output_price_per_1k: 0.0105,
                });
                pricing.insert("gemini-1.5-flash".to_string(), ModelPricing {
                    input_price_per_1k: 0.00035,
                    output_price_per_1k: 0.00105,
                });
                pricing.insert("gemini-pro".to_string(), ModelPricing {
                    input_price_per_1k: 0.0005,
                    output_price_per_1k: 0.0015,
                });
                pricing
            },
            rate_limits: RateLimits {
                requests_per_minute: 60,
                tokens_per_minute: 100000,
            },
            enabled: true,
        });
        
        // Local provider configuration
        providers.insert("local".to_string(), ProviderInfo {
            available_models: vec![
                "llama2".to_string(),
                "mistral".to_string(),
                "codellama".to_string(),
                "vicuna".to_string(),
            ],
            default_model: "llama2".to_string(),
            pricing: HashMap::new(), // Local models are free
            rate_limits: RateLimits {
                requests_per_minute: 120,
                tokens_per_minute: 1000000, // High limit for local
            },
            enabled: false, // Disabled by default
        });
        
        Self {
            providers,
            defaults: DefaultSettings {
                temperature: 0.7,
                max_tokens: 1000,
                timeout_seconds: 120,
                max_concurrent: 5,
                retry_attempts: 3,
            },
            analysis: AnalysisSettings {
                enable_similarity_analysis: true,
                enable_content_analysis: true,
                enable_quality_assessment: true,
                similarity_threshold: 0.7,
                max_keywords: 10,
            },
        }
    }
}

pub async fn load_config() -> Result<EvalEdsConfig> {
    // Follow PromptEds hierarchy: CLI flags > Env vars > Project config > User config > Defaults
    let mut config = EvalEdsConfig::default();
    
    // 1. Load user config
    if let Ok(user_config_path) = get_user_config_path() {
        if user_config_path.exists() {
            let user_config = load_config_from_file(&user_config_path).await?;
            config = merge_configs(config, user_config);
        }
    }
    
    // 2. Load project config (if in a project directory)
    if let Ok(project_config_path) = get_project_config_path() {
        if project_config_path.exists() {
            let project_config = load_config_from_file(&project_config_path).await?;
            config = merge_configs(config, project_config);
        }
    }
    
    // 3. Environment variable overrides
    config = apply_env_overrides(config)?;
    
    Ok(config)
}

async fn load_config_from_file(path: &PathBuf) -> Result<EvalEdsConfig> {
    let config_content = tokio::fs::read_to_string(path).await
        .map_err(|e| EvalError::IoError(e))?;
    
    toml::from_str(&config_content)
        .map_err(|e| EvalError::ConfigError(format!("Invalid config file {}: {}", path.display(), e)))
}

fn merge_configs(base: EvalEdsConfig, override_config: EvalEdsConfig) -> EvalEdsConfig {
    EvalEdsConfig {
        providers: {
            let mut providers = base.providers;
            for (key, value) in override_config.providers {
                providers.insert(key, value);
            }
            providers
        },
        defaults: override_config.defaults, // Replace entirely
        analysis: override_config.analysis, // Replace entirely
    }
}

fn apply_env_overrides(mut config: EvalEdsConfig) -> Result<EvalEdsConfig> {
    // Override with environment variables following PromptEds patterns
    if let Ok(max_concurrent) = std::env::var("EVALEDS_MAX_CONCURRENT") {
        if let Ok(value) = max_concurrent.parse::<u32>() {
            config.defaults.max_concurrent = value;
        }
    }
    
    if let Ok(timeout) = std::env::var("EVALEDS_TIMEOUT") {
        if let Ok(value) = timeout.parse::<u64>() {
            config.defaults.timeout_seconds = value;
        }
    }
    
    if let Ok(temperature) = std::env::var("EVALEDS_TEMPERATURE") {
        if let Ok(value) = temperature.parse::<f32>() {
            config.defaults.temperature = value;
        }
    }
    
    Ok(config)
}

pub async fn save_config(config: &EvalEdsConfig) -> Result<()> {
    let config_path = get_config_path()?;
    
    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| EvalError::IoError(e))?;
    }
    
    let config_content = toml::to_string_pretty(config)
        .map_err(|e| EvalError::SerializationError(e.to_string()))?;
    
    tokio::fs::write(&config_path, config_content).await
        .map_err(|e| EvalError::IoError(e))?;
    
    Ok(())
}

/// Get user config path (following PromptEds XDG pattern)
pub fn get_user_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"));
    
    Ok(config_dir.join("evaleds").join("config.toml"))
}

/// Get project config path (for project-specific settings)
pub fn get_project_config_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .map_err(|e| EvalError::IoError(e))?;
    
    Ok(current_dir.join(".evaleds.toml"))
}

/// Get legacy config path for backward compatibility
pub fn get_legacy_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?;
    
    Ok(home_dir.join(".evaleds").join("config.toml"))
}

/// Get config path (primary user config)
pub fn get_config_path() -> Result<PathBuf> {
    get_user_config_path()
}

pub async fn load_provider_configs() -> Result<HashMap<String, ProviderInfo>> {
    let config = load_config().await?;
    Ok(config.providers)
}

pub async fn get_provider_info(provider_name: &str) -> Result<ProviderInfo> {
    let providers = load_provider_configs().await?;
    providers.get(provider_name)
        .cloned()
        .ok_or_else(|| EvalError::NotFound(format!("Provider '{}' not found", provider_name)))
}

pub async fn update_provider_config(provider_name: &str, provider_info: ProviderInfo) -> Result<()> {
    let mut config = load_config().await?;
    config.providers.insert(provider_name.to_string(), provider_info);
    save_config(&config).await
}

pub async fn enable_provider(provider_name: &str) -> Result<()> {
    let mut config = load_config().await?;
    if let Some(provider) = config.providers.get_mut(provider_name) {
        provider.enabled = true;
        save_config(&config).await
    } else {
        Err(EvalError::NotFound(format!("Provider '{}' not found", provider_name)))
    }
}

pub async fn disable_provider(provider_name: &str) -> Result<()> {
    let mut config = load_config().await?;
    if let Some(provider) = config.providers.get_mut(provider_name) {
        provider.enabled = false;
        save_config(&config).await
    } else {
        Err(EvalError::NotFound(format!("Provider '{}' not found", provider_name)))
    }
}