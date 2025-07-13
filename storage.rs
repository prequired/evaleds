// SQLite storage implementation for evaluations and results
use crate::core::evaluation::*;
use crate::utils::error::{Result, EvalError};
use sqlx::{SqlitePool, Row};
use std::path::Path;

pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Ensure the database directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| EvalError::IoError(e))?;
        }
        
        let database_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&database_url).await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        let storage = Self { pool };
        storage.run_migrations().await?;
        
        Ok(storage)
    }
    
    fn get_db_path() -> Result<std::path::PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| EvalError::ConfigError("Could not find home directory".to_string()))?;
        
        Ok(home_dir.join(".evaleds").join("evaluations.db"))
    }
    
    async fn run_migrations(&self) -> Result<()> {
        // Create evaluations table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS evaluations (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                config TEXT NOT NULL,
                results TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                status TEXT NOT NULL
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        // Create execution_results table for better querying
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS execution_results (
                id TEXT PRIMARY KEY,
                evaluation_id TEXT NOT NULL,
                prompt_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                input TEXT NOT NULL,
                output TEXT NOT NULL,
                status TEXT NOT NULL,
                response_time_ms INTEGER NOT NULL,
                token_count_input INTEGER NOT NULL,
                token_count_output INTEGER NOT NULL,
                cost_usd REAL NOT NULL,
                timestamp TEXT NOT NULL,
                error TEXT,
                FOREIGN KEY (evaluation_id) REFERENCES evaluations (id)
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_evaluations_name ON evaluations (name)")
            .execute(&self.pool)
            .await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_execution_results_evaluation_id ON execution_results (evaluation_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_execution_results_provider ON execution_results (provider)")
            .execute(&self.pool)
            .await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn save_evaluation(&self, evaluation: &Evaluation) -> Result<()> {
        let config_json = serde_json::to_string(&evaluation.config)
            .map_err(|e| EvalError::SerializationError(e.to_string()))?;
        
        let results_json = if let Some(results) = &evaluation.results {
            Some(serde_json::to_string(results)
                .map_err(|e| EvalError::SerializationError(e.to_string()))?)
        } else {
            None
        };
        
        sqlx::query(r#"
            INSERT OR REPLACE INTO evaluations 
            (id, name, description, config, results, created_at, completed_at, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&evaluation.id)
        .bind(&evaluation.name)
        .bind(&evaluation.description)
        .bind(&config_json)
        .bind(&results_json)
        .bind(evaluation.created_at.to_rfc3339())
        .bind(evaluation.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(serde_json::to_string(&evaluation.status).unwrap())
        .execute(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        // Save execution results separately for better querying
        if let Some(results) = &evaluation.results {
            self.save_execution_results(&evaluation.id, &results.executions).await?;
        }
        
        Ok(())
    }
    
    pub async fn load_evaluation(&self, name: &str) -> Result<Option<Evaluation>> {
        let row = sqlx::query(r#"
            SELECT id, name, description, config, results, created_at, completed_at, status
            FROM evaluations 
            WHERE name = ?
        "#)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        if let Some(row) = row {
            let config: EvaluationConfig = serde_json::from_str(row.get("config"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?;
            
            let results: Option<EvaluationResults> = if let Some(results_str): Option<String> = row.get("results") {
                Some(serde_json::from_str(&results_str)
                    .map_err(|e| EvalError::SerializationError(e.to_string()))?)
            } else {
                None
            };
            
            let created_at = chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?
                .with_timezone(&chrono::Utc);
            
            let completed_at = if let Some(completed_str): Option<String> = row.get("completed_at") {
                Some(chrono::DateTime::parse_from_rfc3339(&completed_str)
                    .map_err(|e| EvalError::SerializationError(e.to_string()))?
                    .with_timezone(&chrono::Utc))
            } else {
                None
            };
            
            let status: EvaluationStatus = serde_json::from_str(row.get("status"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?;
            
            Ok(Some(Evaluation {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                config,
                results,
                created_at,
                completed_at,
                status,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn update_evaluation(&self, evaluation: &Evaluation) -> Result<()> {
        self.save_evaluation(evaluation).await
    }
    
    pub async fn list_evaluations(&self) -> Result<Vec<EvaluationSummary>> {
        let rows = sqlx::query(r#"
            SELECT id, name, description, created_at, completed_at, status,
                   (SELECT COUNT(*) FROM execution_results WHERE evaluation_id = evaluations.id) as execution_count
            FROM evaluations 
            ORDER BY created_at DESC
        "#)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        let mut summaries = Vec::new();
        
        for row in rows {
            let created_at = chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?
                .with_timezone(&chrono::Utc);
            
            let completed_at = if let Some(completed_str): Option<String> = row.get("completed_at") {
                Some(chrono::DateTime::parse_from_rfc3339(&completed_str)
                    .map_err(|e| EvalError::SerializationError(e.to_string()))?
                    .with_timezone(&chrono::Utc))
            } else {
                None
            };
            
            let status: EvaluationStatus = serde_json::from_str(row.get("status"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?;
            
            summaries.push(EvaluationSummary {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                status,
                created_at,
                completed_at,
                execution_count: row.get::<i64, _>("execution_count") as u32,
            });
        }
        
        Ok(summaries)
    }
    
    pub async fn delete_evaluation(&self, name: &str) -> Result<bool> {
        // First get the evaluation ID
        let evaluation_id: Option<String> = sqlx::query("SELECT id FROM evaluations WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?
            .map(|row| row.get("id"));
        
        if let Some(id) = evaluation_id {
            // Delete execution results first (foreign key constraint)
            sqlx::query("DELETE FROM execution_results WHERE evaluation_id = ?")
                .bind(&id)
                .execute(&self.pool)
                .await
                .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
            
            // Delete the evaluation
            let result = sqlx::query("DELETE FROM evaluations WHERE id = ?")
                .bind(&id)
                .execute(&self.pool)
                .await
                .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        } else {
            Ok(false)
        }
    }
    
    async fn save_execution_results(&self, evaluation_id: &str, results: &[ExecutionResult]) -> Result<()> {
        // Delete existing results for this evaluation
        sqlx::query("DELETE FROM execution_results WHERE evaluation_id = ?")
            .bind(evaluation_id)
            .execute(&self.pool)
            .await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        // Insert new results
        for result in results {
            sqlx::query(r#"
                INSERT INTO execution_results 
                (id, evaluation_id, prompt_id, provider, model, input, output, status,
                 response_time_ms, token_count_input, token_count_output, cost_usd, timestamp, error)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&result.id)
            .bind(evaluation_id)
            .bind(&result.prompt_id)
            .bind(&result.provider)
            .bind(&result.model)
            .bind(&result.input)
            .bind(&result.output)
            .bind(serde_json::to_string(&result.status).unwrap())
            .bind(result.metadata.response_time_ms as i64)
            .bind(result.metadata.token_count_input as i64)
            .bind(result.metadata.token_count_output as i64)
            .bind(result.metadata.cost_usd)
            .bind(result.metadata.timestamp.to_rfc3339())
            .bind(&result.metadata.error)
            .execute(&self.pool)
            .await
            .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    pub async fn get_evaluation_stats(&self) -> Result<StorageStats> {
        let stats_row = sqlx::query(r#"
            SELECT 
                COUNT(*) as total_evaluations,
                COUNT(CASE WHEN status = '"Completed"' THEN 1 END) as completed_evaluations,
                COUNT(CASE WHEN status = '"Running"' THEN 1 END) as running_evaluations,
                (SELECT COUNT(*) FROM execution_results) as total_executions,
                (SELECT SUM(cost_usd) FROM execution_results) as total_cost
            FROM evaluations
        "#)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        Ok(StorageStats {
            total_evaluations: stats_row.get::<i64, _>("total_evaluations") as u32,
            completed_evaluations: stats_row.get::<i64, _>("completed_evaluations") as u32,
            running_evaluations: stats_row.get::<i64, _>("running_evaluations") as u32,
            total_executions: stats_row.get::<i64, _>("total_executions") as u32,
            total_cost: stats_row.get::<Option<f64>, _>("total_cost").unwrap_or(0.0),
        })
    }
    
    pub async fn search_evaluations(&self, query: &str) -> Result<Vec<EvaluationSummary>> {
        let rows = sqlx::query(r#"
            SELECT id, name, description, created_at, completed_at, status,
                   (SELECT COUNT(*) FROM execution_results WHERE evaluation_id = evaluations.id) as execution_count
            FROM evaluations 
            WHERE name LIKE ? OR description LIKE ?
            ORDER BY created_at DESC
        "#)
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EvalError::DatabaseError(e.to_string()))?;
        
        let mut summaries = Vec::new();
        
        for row in rows {
            let created_at = chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?
                .with_timezone(&chrono::Utc);
            
            let completed_at = if let Some(completed_str): Option<String> = row.get("completed_at") {
                Some(chrono::DateTime::parse_from_rfc3339(&completed_str)
                    .map_err(|e| EvalError::SerializationError(e.to_string()))?
                    .with_timezone(&chrono::Utc))
            } else {
                None
            };
            
            let status: EvaluationStatus = serde_json::from_str(row.get("status"))
                .map_err(|e| EvalError::SerializationError(e.to_string()))?;
            
            summaries.push(EvaluationSummary {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                status,
                created_at,
                completed_at,
                execution_count: row.get::<i64, _>("execution_count") as u32,
            });
        }
        
        Ok(summaries)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvaluationSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: EvaluationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub execution_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageStats {
    pub total_evaluations: u32,
    pub completed_evaluations: u32,
    pub running_evaluations: u32,
    pub total_executions: u32,
    pub total_cost: f64,
}