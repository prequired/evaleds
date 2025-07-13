// Extended evaluation types to support PromptEds alignment
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Extended Evaluation with PromptEds-style metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,                    // PromptEds-style tags
    pub category: Option<String>,             // PromptEds-style category
    pub config: EvaluationConfig,
    pub results: Option<EvaluationResults>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,            // Track updates
    pub completed_at: Option<DateTime<Utc>>,
    pub status: EvaluationStatus,
    pub version: u32,                         // Version tracking
    pub author: Option<String>,               // Creator metadata
}

// Extended EvaluationSummary for list commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub status: EvaluationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_count: u32,
    pub total_cost: f64,
    pub success_rate: f32,
    pub version: u32,
    pub author: Option<String>,
}

// Storage trait for evaluation management
#[async_trait::async_trait]
pub trait EvaluationStorage {
    async fn save_evaluation(&self, evaluation: &Evaluation) -> Result<()>;
    async fn load_evaluation(&self, name: &str) -> Result<Option<Evaluation>>;
    async fn update_evaluation(&self, evaluation: &Evaluation) -> Result<()>;
    async fn delete_evaluation(&self, name: &str) -> Result<bool>;
    async fn list_evaluations(&self) -> Result<Vec<EvaluationSummary>>;
    async fn evaluation_exists(&self, name: &str) -> Result<bool>;
    async fn search_evaluations(&self, query: &str) -> Result<Vec<EvaluationSummary>>;
    async fn filter_evaluations(&self, filters: &EvaluationFilters) -> Result<Vec<EvaluationSummary>>;
}

// Filters for evaluation queries (PromptEds style)
#[derive(Debug, Clone, Default)]
pub struct EvaluationFilters {
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub author: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl Evaluation {
    pub fn new(name: String, config: EvaluationConfig) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            tags: Vec::new(),
            category: None,
            config,
            results: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            status: EvaluationStatus::Configured,
            version: 1,
            author: None,
        }
    }
    
    /// Update the evaluation's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
    
    /// Add a tag if it doesn't already exist
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }
    }
    
    /// Remove a tag if it exists
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.touch();
        }
    }
    
    /// Check if evaluation has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
    
    /// Get formatted display name with tags
    pub fn display_name(&self) -> String {
        if self.tags.is_empty() {
            self.name.clone()
        } else {
            format!("{} #{}", self.name, self.tags.join(" #"))
        }
    }
    
    /// Get summary for list display
    pub fn to_summary(&self) -> EvaluationSummary {
        let (execution_count, total_cost, success_rate) = if let Some(results) = &self.results {
            (
                results.summary.total_executions,
                results.summary.total_cost,
                results.summary.success_rate,
            )
        } else {
            (0, 0.0, 0.0)
        };
        
        EvaluationSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            category: self.category.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
            execution_count,
            total_cost,
            success_rate,
            version: self.version,
            author: self.author.clone(),
        }
    }
}