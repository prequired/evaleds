// PromptEds integration client for seamless prompt management
use crate::utils::error::{Result, EvalError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEdsPrompt {
    pub name: String,
    pub version: u32,
    pub template: String,
    pub variables: Vec<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEdsConfig {
    pub prompts_dir: String,
    pub default_extension: String,
}

pub struct PromptEdsClient {
    config: PromptEdsConfig,
    prompts_cache: HashMap<String, PromptEdsPrompt>,
}

impl PromptEdsClient {
    pub async fn new() -> Result<Self> {
        let config = Self::load_config().await?;
        let mut client = Self {
            config,
            prompts_cache: HashMap::new(),
        };
        
        // Load all prompts into cache
        client.refresh_cache().await?;
        
        Ok(client)
    }
    
    async fn load_config() -> Result<PromptEdsConfig> {
        // Try to load from PromptEds config first
        let prompteds_config_path = dirs::home_dir()
            .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?
            .join(".prompteds")
            .join("config.toml");
        
        if prompteds_config_path.exists() {
            let config_content = tokio::fs::read_to_string(&prompteds_config_path).await
                .map_err(|e| EvalError::IoError(e))?;
            
            let prompteds_config: toml::Value = toml::from_str(&config_content)
                .map_err(|e| EvalError::ConfigError(format!("Invalid PromptEds config: {}", e)))?;
            
            let prompts_dir = prompteds_config
                .get("prompts_dir")
                .and_then(|v| v.as_str())
                .unwrap_or("~/.prompteds/prompts")
                .to_string();
            
            return Ok(PromptEdsConfig {
                prompts_dir: Self::expand_path(&prompts_dir)?,
                default_extension: "md".to_string(),
            });
        }
        
        // Fallback to default configuration
        let home_dir = dirs::home_dir()
            .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?;
        
        Ok(PromptEdsConfig {
            prompts_dir: home_dir.join(".prompteds").join("prompts").to_string_lossy().to_string(),
            default_extension: "md".to_string(),
        })
    }
    
    fn expand_path(path: &str) -> Result<String> {
        if path.starts_with("~/") {
            let home_dir = dirs::home_dir()
                .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?;
            Ok(home_dir.join(&path[2..]).to_string_lossy().to_string())
        } else {
            Ok(path.to_string())
        }
    }
    
    async fn refresh_cache(&mut self) -> Result<()> {
        self.prompts_cache.clear();
        
        let prompts_dir = Path::new(&self.config.prompts_dir);
        if !prompts_dir.exists() {
            return Ok(()); // No prompts directory, nothing to cache
        }
        
        let mut entries = tokio::fs::read_dir(prompts_dir).await
            .map_err(|e| EvalError::IoError(e))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| EvalError::IoError(e))? {
            
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(&format!(".{}", self.config.default_extension)) {
                    let prompt_name = file_name.trim_end_matches(&format!(".{}", self.config.default_extension));
                    
                    match self.load_prompt_file(&entry.path()).await {
                        Ok(prompt) => {
                            self.prompts_cache.insert(prompt_name.to_string(), prompt);
                        },
                        Err(e) => {
                            eprintln!("Warning: Failed to load prompt '{}': {}", prompt_name, e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn load_prompt_file(&self, path: &Path) -> Result<PromptEdsPrompt> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| EvalError::IoError(e))?;
        
        let metadata = tokio::fs::metadata(path).await
            .map_err(|e| EvalError::IoError(e))?;
        
        let file_name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| EvalError::ConfigError("Invalid file name".to_string()))?;
        
        // Parse frontmatter if present
        let (frontmatter, template) = self.parse_frontmatter(&content);
        
        // Extract variables from template
        let variables = self.extract_variables(&template);
        
        let created_time = metadata.created()
            .or_else(|_| metadata.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        
        let modified_time = metadata.modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        
        Ok(PromptEdsPrompt {
            name: file_name.to_string(),
            version: frontmatter.get("version")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            template,
            variables,
            tags: frontmatter.get("tags")
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            description: frontmatter.get("description").cloned(),
            created_at: chrono::DateTime::from(created_time),
            updated_at: chrono::DateTime::from(modified_time),
        })
    }
    
    fn parse_frontmatter(&self, content: &str) -> (HashMap<String, String>, String) {
        if content.starts_with("---") {
            if let Some(end_pos) = content[3..].find("---") {
                let frontmatter_content = &content[3..end_pos + 3];
                let template_content = &content[end_pos + 6..].trim_start();
                
                let mut frontmatter = HashMap::new();
                for line in frontmatter_content.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        frontmatter.insert(
                            key.trim().to_string(),
                            value.trim().trim_matches('"').to_string(),
                        );
                    }
                }
                
                return (frontmatter, template_content.to_string());
            }
        }
        
        (HashMap::new(), content.to_string())
    }
    
    fn extract_variables(&self, template: &str) -> Vec<String> {
        let mut variables = std::collections::HashSet::new();
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        
        for cap in re.captures_iter(template) {
            if let Some(var) = cap.get(1) {
                variables.insert(var.as_str().to_string());
            }
        }
        
        let mut vars: Vec<_> = variables.into_iter().collect();
        vars.sort();
        vars
    }
    
    pub async fn list_prompts(&self) -> Result<Vec<PromptEdsPrompt>> {
        Ok(self.prompts_cache.values().cloned().collect())
    }
    
    pub async fn get_prompt(&self, name: &str) -> Result<PromptEdsPrompt> {
        self.prompts_cache.get(name)
            .cloned()
            .ok_or_else(|| EvalError::NotFound(format!("Prompt '{}' not found", name)))
    }
    
    pub async fn get_prompt_content(&self, name: &str, variables: &HashMap<String, String>) -> Result<String> {
        let prompt = self.get_prompt(name).await?;
        Ok(self.render_template(&prompt.template, variables))
    }
    
    fn render_template(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut rendered = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
        
        rendered
    }
    
    pub async fn search_prompts(&self, query: &str) -> Result<Vec<PromptEdsPrompt>> {
        let query_lower = query.to_lowercase();
        
        let matching_prompts: Vec<_> = self.prompts_cache.values()
            .filter(|prompt| {
                prompt.name.to_lowercase().contains(&query_lower)
                    || prompt.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                    || prompt.description.as_ref()
                        .map(|desc| desc.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();
        
        Ok(matching_prompts)
    }
    
    pub async fn get_prompts_by_tag(&self, tag: &str) -> Result<Vec<PromptEdsPrompt>> {
        let tag_lower = tag.to_lowercase();
        
        let tagged_prompts: Vec<_> = self.prompts_cache.values()
            .filter(|prompt| {
                prompt.tags.iter().any(|t| t.to_lowercase() == tag_lower)
            })
            .cloned()
            .collect();
        
        Ok(tagged_prompts)
    }
    
    pub async fn validate_prompt(&self, name: &str, variables: &HashMap<String, String>) -> Result<ValidationResult> {
        let prompt = self.get_prompt(name).await?;
        
        let mut missing_variables = Vec::new();
        let mut extra_variables = Vec::new();
        
        // Check for missing required variables
        for required_var in &prompt.variables {
            if !variables.contains_key(required_var) {
                missing_variables.push(required_var.clone());
            }
        }
        
        // Check for extra variables
        for provided_var in variables.keys() {
            if !prompt.variables.contains(provided_var) {
                extra_variables.push(provided_var.clone());
            }
        }
        
        let is_valid = missing_variables.is_empty();
        
        Ok(ValidationResult {
            is_valid,
            missing_variables,
            extra_variables,
            rendered_preview: if is_valid {
                Some(self.render_template(&prompt.template, variables))
            } else {
                None
            },
        })
    }
    
    pub async fn get_prompt_stats(&self) -> Result<PromptStats> {
        let total_prompts = self.prompts_cache.len();
        
        let mut tags = std::collections::HashSet::new();
        let mut total_variables = 0;
        let mut recent_prompts = 0;
        
        let week_ago = chrono::Utc::now() - chrono::Duration::days(7);
        
        for prompt in self.prompts_cache.values() {
            tags.extend(prompt.tags.iter().cloned());
            total_variables += prompt.variables.len();
            
            if prompt.updated_at > week_ago {
                recent_prompts += 1;
            }
        }
        
        Ok(PromptStats {
            total_prompts,
            unique_tags: tags.len(),
            total_variables,
            recent_prompts,
            avg_variables_per_prompt: if total_prompts > 0 {
                total_variables as f32 / total_prompts as f32
            } else {
                0.0
            },
        })
    }
    
    pub async fn refresh(&mut self) -> Result<()> {
        self.refresh_cache().await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub missing_variables: Vec<String>,
    pub extra_variables: Vec<String>,
    pub rendered_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptStats {
    pub total_prompts: usize,
    pub unique_tags: usize,
    pub total_variables: usize,
    pub recent_prompts: usize,
    pub avg_variables_per_prompt: f32,
}

// Mock implementation for when PromptEds is not available
pub struct MockPromptEdsClient;

impl MockPromptEdsClient {
    pub async fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub async fn list_prompts(&self) -> Result<Vec<PromptEdsPrompt>> {
        Ok(vec![
            PromptEdsPrompt {
                name: "data-analysis".to_string(),
                version: 1,
                template: "Analyze the following data: {{data}}\n\nGoal: {{goal}}".to_string(),
                variables: vec!["data".to_string(), "goal".to_string()],
                tags: vec!["analysis".to_string(), "data".to_string()],
                description: Some("Template for data analysis tasks".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            PromptEdsPrompt {
                name: "code-review".to_string(),
                version: 1,
                template: "Review the following {{language}} code:\n\n{{code}}\n\nFocus on: {{focus_areas}}".to_string(),
                variables: vec!["language".to_string(), "code".to_string(), "focus_areas".to_string()],
                tags: vec!["code".to_string(), "review".to_string()],
                description: Some("Template for code review tasks".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }
    
    pub async fn get_prompt(&self, name: &str) -> Result<PromptEdsPrompt> {
        let prompts = self.list_prompts().await?;
        prompts.into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| EvalError::NotFound(format!("Prompt '{}' not found", name)))
    }
    
    pub async fn get_prompt_content(&self, name: &str, variables: &HashMap<String, String>) -> Result<String> {
        let prompt = self.get_prompt(name).await?;
        let mut rendered = prompt.template;
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
        
        Ok(rendered)
    }
}