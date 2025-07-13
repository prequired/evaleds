// Provider Manager and individual provider implementations
use crate::core::evaluation::{ModelSettings, ExecutionResult, ExecutionMetadata, ExecutionStatus};
use crate::utils::error::Result;
use async_trait::async_trait;
use std::time::Instant;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Provider: Send + Sync {
    async fn execute(&self, prompt: &str, model: &str, settings: &ModelSettings) -> Result<ProviderResponse>;
    fn get_models(&self) -> Vec<String>;
    fn estimate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64;
    fn supports_streaming(&self) -> bool { false }
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
    pub usage: Usage,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

pub struct ProviderManager {
    providers: HashMap<String, Box<dyn Provider>>,
}

impl ProviderManager {
    pub async fn new() -> Result<Self> {
        let mut providers: HashMap<String, Box<dyn Provider>> = HashMap::new();
        
        // Initialize providers
        if let Ok(openai_provider) = OpenAIProvider::new().await {
            providers.insert("openai".to_string(), Box::new(openai_provider));
        }
        
        if let Ok(anthropic_provider) = AnthropicProvider::new().await {
            providers.insert("anthropic".to_string(), Box::new(anthropic_provider));
        }
        
        if let Ok(google_provider) = GoogleProvider::new().await {
            providers.insert("google".to_string(), Box::new(google_provider));
        }
        
        if let Ok(local_provider) = LocalProvider::new().await {
            providers.insert("local".to_string(), Box::new(local_provider));
        }
        
        Ok(Self { providers })
    }
    
    pub async fn execute_prompt(
        &self,
        provider_name: &str,
        model: &str,
        prompt: &str,
        settings: &ModelSettings,
    ) -> Result<ExecutionResult> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| crate::utils::error::EvalError::ProviderError(
                format!("Provider '{}' not available", provider_name)
            ))?;
        
        let start_time = Instant::now();
        
        match provider.execute(prompt, model, settings).await {
            Ok(response) => {
                let response_time = start_time.elapsed();
                let cost = provider.estimate_cost(model, response.usage.input_tokens, response.usage.output_tokens);
                
                Ok(ExecutionResult {
                    id: uuid::Uuid::new_v4().to_string(),
                    prompt_id: uuid::Uuid::new_v4().to_string(),
                    provider: provider_name.to_string(),
                    model: model.to_string(),
                    input: prompt.to_string(),
                    output: response.content,
                    metadata: ExecutionMetadata {
                        response_time_ms: response_time.as_millis() as u64,
                        token_count_input: response.usage.input_tokens,
                        token_count_output: response.usage.output_tokens,
                        cost_usd: cost,
                        timestamp: chrono::Utc::now(),
                        error: None,
                        rate_limit_info: None,
                    },
                    status: ExecutionStatus::Success,
                })
            },
            Err(e) => {
                let response_time = start_time.elapsed();
                
                Ok(ExecutionResult {
                    id: uuid::Uuid::new_v4().to_string(),
                    prompt_id: uuid::Uuid::new_v4().to_string(),
                    provider: provider_name.to_string(),
                    model: model.to_string(),
                    input: prompt.to_string(),
                    output: String::new(),
                    metadata: ExecutionMetadata {
                        response_time_ms: response_time.as_millis() as u64,
                        token_count_input: 0,
                        token_count_output: 0,
                        cost_usd: 0.0,
                        timestamp: chrono::Utc::now(),
                        error: Some(e.to_string()),
                        rate_limit_info: None,
                    },
                    status: ExecutionStatus::Failed,
                })
            }
        }
    }
    
    pub fn get_available_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    pub fn get_provider_models(&self, provider_name: &str) -> Option<Vec<String>> {
        self.providers.get(provider_name).map(|p| p.get_models())
    }
}

// OpenAI Provider Implementation
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
}

impl OpenAIProvider {
    pub async fn new() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| crate::utils::error::EvalError::ConfigError(
                "OPENAI_API_KEY environment variable not found".to_string()
            ))?;
        
        let client = reqwest::Client::new();
        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn execute(&self, prompt: &str, model: &str, settings: &ModelSettings) -> Result<ProviderResponse> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": settings.temperature.unwrap_or(0.7),
            "max_tokens": settings.max_tokens.unwrap_or(1000),
        });
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let usage = &response_json["usage"];
        let input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0) as u32;
        
        Ok(ProviderResponse {
            content,
            usage: Usage {
                input_tokens,
                output_tokens,
                total_tokens: input_tokens + output_tokens,
            },
            metadata: HashMap::new(),
        })
    }
    
    fn get_models(&self) -> Vec<String> {
        vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]
    }
    
    fn estimate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        let pricing = match model {
            "gpt-4" => (0.03, 0.06),
            "gpt-4-turbo" => (0.01, 0.03),
            "gpt-3.5-turbo" => (0.001, 0.002),
            _ => (0.01, 0.03),
        };
        
        let input_cost = (input_tokens as f64 / 1000.0) * pricing.0;
        let output_cost = (output_tokens as f64 / 1000.0) * pricing.1;
        
        input_cost + output_cost
    }
}

// Anthropic Provider Implementation
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicProvider {
    pub async fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| crate::utils::error::EvalError::ConfigError(
                "ANTHROPIC_API_KEY environment variable not found".to_string()
            ))?;
        
        let client = reqwest::Client::new();
        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn execute(&self, prompt: &str, model: &str, settings: &ModelSettings) -> Result<ProviderResponse> {
        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": settings.max_tokens.unwrap_or(1000),
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let content = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let usage = &response_json["usage"];
        let input_tokens = usage["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = usage["output_tokens"].as_u64().unwrap_or(0) as u32;
        
        Ok(ProviderResponse {
            content,
            usage: Usage {
                input_tokens,
                output_tokens,
                total_tokens: input_tokens + output_tokens,
            },
            metadata: HashMap::new(),
        })
    }
    
    fn get_models(&self) -> Vec<String> {
        vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]
    }
    
    fn estimate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        let pricing = match model {
            "claude-3-opus-20240229" => (0.015, 0.075),
            "claude-3-sonnet-20240229" => (0.003, 0.015),
            "claude-3-haiku-20240307" => (0.00025, 0.00125),
            _ => (0.003, 0.015),
        };
        
        let input_cost = (input_tokens as f64 / 1000.0) * pricing.0;
        let output_cost = (output_tokens as f64 / 1000.0) * pricing.1;
        
        input_cost + output_cost
    }
}

// Google Provider Implementation
pub struct GoogleProvider {
    client: reqwest::Client,
    api_key: String,
}

impl GoogleProvider {
    pub async fn new() -> Result<Self> {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .map_err(|_| crate::utils::error::EvalError::ConfigError(
                "GOOGLE_API_KEY environment variable not found".to_string()
            ))?;
        
        let client = reqwest::Client::new();
        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn execute(&self, prompt: &str, model: &str, settings: &ModelSettings) -> Result<ProviderResponse> {
        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "temperature": settings.temperature.unwrap_or(0.7),
                "maxOutputTokens": settings.max_tokens.unwrap_or(1000),
            }
        });
        
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", 
            model, self.api_key);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        // Google doesn't provide detailed token usage in the response
        let estimated_input_tokens = (prompt.len() / 4) as u32; // Rough estimation
        let estimated_output_tokens = (content.len() / 4) as u32;
        
        Ok(ProviderResponse {
            content,
            usage: Usage {
                input_tokens: estimated_input_tokens,
                output_tokens: estimated_output_tokens,
                total_tokens: estimated_input_tokens + estimated_output_tokens,
            },
            metadata: HashMap::new(),
        })
    }
    
    fn get_models(&self) -> Vec<String> {
        vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-pro".to_string(),
        ]
    }
    
    fn estimate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        let pricing = match model {
            "gemini-1.5-pro" => (0.0035, 0.0105),
            "gemini-1.5-flash" => (0.00035, 0.00105),
            "gemini-pro" => (0.0005, 0.0015),
            _ => (0.0005, 0.0015),
        };
        
        let input_cost = (input_tokens as f64 / 1000.0) * pricing.0;
        let output_cost = (output_tokens as f64 / 1000.0) * pricing.1;
        
        input_cost + output_cost
    }
}

// Local Provider Implementation (for local models like Ollama)
pub struct LocalProvider {
    client: reqwest::Client,
    base_url: String,
}

impl LocalProvider {
    pub async fn new() -> Result<Self> {
        let base_url = std::env::var("LOCAL_MODEL_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        
        let client = reqwest::Client::new();
        Ok(Self { client, base_url })
    }
}

#[async_trait]
impl Provider for LocalProvider {
    async fn execute(&self, prompt: &str, model: &str, settings: &ModelSettings) -> Result<ProviderResponse> {
        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": settings.temperature.unwrap_or(0.7),
                "num_predict": settings.max_tokens.unwrap_or(1000),
            }
        });
        
        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| crate::utils::error::EvalError::ProviderError(e.to_string()))?;
        
        let content = response_json["response"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        // Local models typically don't provide token counts
        let estimated_input_tokens = (prompt.len() / 4) as u32;
        let estimated_output_tokens = (content.len() / 4) as u32;
        
        Ok(ProviderResponse {
            content,
            usage: Usage {
                input_tokens: estimated_input_tokens,
                output_tokens: estimated_output_tokens,
                total_tokens: estimated_input_tokens + estimated_output_tokens,
            },
            metadata: HashMap::new(),
        })
    }
    
    fn get_models(&self) -> Vec<String> {
        vec![
            "llama2".to_string(),
            "mistral".to_string(),
            "codellama".to_string(),
            "vicuna".to_string(),
        ]
    }
    
    fn estimate_cost(&self, _model: &str, _input_tokens: u32, _output_tokens: u32) -> f64 {
        0.0 // Local models are free
    }
}