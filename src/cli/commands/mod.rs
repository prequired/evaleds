// Command implementations aligned with PromptEds patterns
use crate::cli::args::*;
use crate::core::evaluation::*;
use crate::core::storage::Storage;
use crate::utils::{error::*, config::load_config};
use crate::ui::{interactive_style::*, output_formatting::*};

// CREATE COMMAND (aligned with PromptEds create)
pub mod create {
    use super::*;
    
    pub async fn execute(args: CreateArgs) -> Result<()> {
        let config = load_config().await?;
        let storage = Storage::new().await?;
        
        // Check if evaluation already exists (PromptEds pattern)
        if storage.evaluation_exists(&args.name).await? {
            return Err(EvalError::AlreadyExists(args.name));
        }
        
        if args.interactive {
            return execute_interactive(args, config).await;
        }
        
        // Non-interactive creation (minimal setup like PromptEds)
        let evaluation = Evaluation::new(args.name.clone(), EvaluationConfig::default());
        let mut evaluation = evaluation;
        
        // Apply command line arguments
        if let Some(description) = args.description {
            evaluation.description = Some(description);
        }
        
        // Add tags (PromptEds style)
        if !args.tags.is_empty() {
            evaluation.tags = args.tags;
        }
        
        if let Some(category) = args.category {
            evaluation.category = Some(category);
        }
        
        storage.save_evaluation(&evaluation).await?;
        
        display_completion("Created evaluation", &args.name);
        display_next_steps(&args.name);
        
        Ok(())
    }
    
    async fn execute_interactive(args: CreateArgs, _config: EvalEdsConfig) -> Result<()> {
        display_interactive_header("EvalEds Interactive Evaluation Creator");
        
        // Get evaluation name (if not provided or validate if provided)
        let name = if args.name.is_empty() {
            prompt_text("Evaluation name", None, true)?
        } else {
            args.name.clone()
        };
        
        // Description (PromptEds style)
        let description = if let Some(desc) = args.description {
            Some(desc)
        } else {
            let desc = prompt_text("Description (optional)", None, false)?;
            if desc.trim().is_empty() { None } else { Some(desc) }
        };
        
        // Category (PromptEds style)
        let category = if let Some(cat) = args.category {
            Some(cat)
        } else {
            let cat = prompt_text("Category (optional)", None, false)?;
            if cat.trim().is_empty() { None } else { Some(cat) }
        };
        
        // Tags (PromptEds style)
        let tags = if !args.tags.is_empty() {
            args.tags
        } else {
            let tags_str = prompt_text("Tags (comma-separated, optional)", None, false)?;
            if tags_str.trim().is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.trim().to_string()).collect()
            }
        };
        
        display_step_header(1, "Prompt Configuration");
        let prompts = setup_prompts_interactive().await?;
        
        display_step_header(2, "Provider Configuration");
        let providers = setup_providers_interactive().await?;
        
        display_step_header(3, "Analysis Configuration");
        let analysis_options = setup_analysis_interactive().await?;
        
        // Create evaluation
        let mut evaluation = Evaluation::new(name.clone(), EvaluationConfig {
            prompts,
            providers,
            analysis_options,
            batch_settings: BatchSettings::default(),
        });
        
        evaluation.description = description;
        evaluation.category = category;
        evaluation.tags = tags;
        
        // Display metadata summary (PromptEds style)
        display_metadata(&name, evaluation.description.as_deref(), evaluation.category.as_deref());
        display_tags(&evaluation.tags);
        
        let storage = Storage::new().await?;
        storage.save_evaluation(&evaluation).await?;
        
        display_completion("Created evaluation", &name);
        display_next_steps(&name);
        
        Ok(())
    }
    
    async fn setup_prompts_interactive() -> Result<Vec<PromptConfig>> {
        // Implementation here...
        Ok(Vec::new())
    }
    
    async fn setup_providers_interactive() -> Result<Vec<ProviderConfig>> {
        // Implementation here...
        Ok(Vec::new())
    }
    
    async fn setup_analysis_interactive() -> Result<AnalysisOptions> {
        let selections = prompt_analysis_features()?;
        
        Ok(AnalysisOptions {
            response_metrics: selections.contains(&0),
            similarity_analysis: selections.contains(&1),
            content_analysis: selections.contains(&2),
            quality_indicators: selections.contains(&3),
            cost_analysis: selections.contains(&4),
            performance_analysis: selections.contains(&5),
        })
    }
}

// RUN COMMAND (aligned with PromptEds run)
pub mod run {
    use super::*;
    
    pub async fn execute(args: RunArgs) -> Result<()> {
        let storage = Storage::new().await?;
        let evaluation = storage.load_evaluation(&args.name).await?
            .ok_or_else(|| EvalError::NotFound(args.name.clone()))?;
        
        // Check if already completed (PromptEds pattern)
        if matches!(evaluation.status, EvaluationStatus::Completed) && !args.force {
            display_warning(&format!("Evaluation '{}' already completed. Use --force to re-run.", args.name));
            return Ok(());
        }
        
        if args.dry_run {
            return execute_dry_run(&evaluation);
        }
        
        // Execute the evaluation
        println!("🚀 Running evaluation: {}", format_evaluation_name(&evaluation.name, true));
        
        // Implementation would continue here...
        
        display_completion("Completed evaluation", &args.name);
        display_info(&format!("View results: evaleds show {}", args.name));
        
        Ok(())
    }
    
    fn execute_dry_run(evaluation: &Evaluation) -> Result<()> {
        println!("🔍 Dry run for evaluation: {}", format_evaluation_name(&evaluation.name, true));
        
        // Show what would be executed
        let total_executions = calculate_total_executions(&evaluation.config);
        
        println!("📊 Would execute {} total runs:", format_metric(&total_executions.to_string(), true));
        
        for prompt_config in &evaluation.config.prompts {
            let variations = 1 + prompt_config.variations.len();
            println!("  📝 Prompt variations: {}", variations);
        }
        
        for provider_config in &evaluation.config.providers {
            println!("  🤖 {}: {} models", provider_config.name, provider_config.models.len());
        }
        
        Ok(())
    }
    
    fn calculate_total_executions(config: &EvaluationConfig) -> u32 {
        let mut total = 0;
        
        for prompt_config in &config.prompts {
            let prompt_variations = 1 + prompt_config.variations.len() as u32;
            
            for provider_config in &config.providers {
                total += prompt_variations * provider_config.models.len() as u32;
            }
        }
        
        total
    }
}

// SHOW COMMAND (aligned with PromptEds show)
pub mod show {
    use super::*;
    
    pub async fn execute(args: ShowArgs) -> Result<()> {
        let storage = Storage::new().await?;
        let evaluation = storage.load_evaluation(&args.name).await?
            .ok_or_else(|| EvalError::NotFound(args.name.clone()))?;
        
        let use_color = should_use_colors(false); // Would get from global args
        
        if args.raw {
            // Show raw configuration (PromptEds style)
            let config_json = serde_json::to_string_pretty(&evaluation.config)?;
            println!("{}", config_json);
            return Ok(());
        }
        
        if args.metadata {
            // Show metadata only (PromptEds style)
            display_evaluation_metadata(&evaluation, use_color)?;
            return Ok(());
        }
        
        if args.web {
            // Launch web interface
            return launch_web_interface(&evaluation).await;
        }
        
        if let Some(export_format) = args.export {
            // Export results
            return export_evaluation(&evaluation, &export_format, args.output.as_deref()).await;
        }
        
        // Default: show summary (PromptEds style)
        display_evaluation_metadata(&evaluation, use_color)?;
        display_results_summary(&evaluation, use_color)?;
        
        Ok(())
    }
    
    async fn launch_web_interface(evaluation: &Evaluation) -> Result<()> {
        display_info("Launching web interface...");
        
        // Implementation would start web server here
        // let server = WebServer::new(evaluation, 0).await?;
        // let url = server.start().await?;
        
        let url = "http://localhost:8080"; // Placeholder
        println!("🌐 Web interface available at: {}", format_metric(url, true));
        
        // Open browser if available
        if let Err(_) = webbrowser::open(url) {
            display_info("Open the URL above in your browser to view results");
        }
        
        Ok(())
    }
    
    async fn export_evaluation(evaluation: &Evaluation, format: &str, output: Option<&str>) -> Result<()> {
        let output_path = output.unwrap_or(&format!("{}.{}", evaluation.name, format));
        
        match format {
            "markdown" | "md" => {
                // Generate markdown report
                display_info(&format!("Exporting to markdown: {}", output_path));
                // Implementation here...
            },
            "json" => {
                // Export as JSON
                display_info(&format!("Exporting to JSON: {}", output_path));
                let json = serde_json::to_string_pretty(evaluation)?;
                tokio::fs::write(output_path, json).await?;
            },
            "html" => {
                // Generate HTML report
                display_info(&format!("Exporting to HTML: {}", output_path));
                // Implementation here...
            },
            _ => {
                return Err(EvalError::ValidationError(format!("Unsupported export format: {}", format)));
            }
        }
        
        display_success(&format!("Exported to {}", output_path));
        Ok(())
    }
}

// LIST COMMAND (aligned with PromptEds list)
pub mod list {
    use super::*;
    
    pub async fn execute(args: ListArgs) -> Result<()> {
        let storage = Storage::new().await?;
        
        // Build filters (PromptEds pattern)
        let mut evaluations = storage.list_evaluations().await?;
        
        // Apply filters
        if !args.tags.is_empty() {
            evaluations.retain(|eval| {
                args.tags.iter().any(|tag| eval.tags.contains(tag))
            });
        }
        
        if let Some(category) = &args.category {
            evaluations.retain(|eval| eval.category.as_ref() == Some(category));
        }
        
        if let Some(status) = &args.status {
            evaluations.retain(|eval| format!("{:?}", eval.status).to_lowercase() == status.to_lowercase());
        }
        
        // Apply sorting (PromptEds pattern)
        match args.sort.as_str() {
            "name" => evaluations.sort_by(|a, b| a.name.cmp(&b.name)),
            "created" => evaluations.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            "updated" => evaluations.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
            "status" => evaluations.sort_by(|a, b| format!("{:?}", a.status).cmp(&format!("{:?}", b.status))),
            _ => {
                display_warning(&format!("Unknown sort field '{}', using 'name'", args.sort));
                evaluations.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        
        if args.reverse {
            evaluations.reverse();
        }
        
        // Display results (PromptEds style)
        if evaluations.is_empty() {
            println!("No evaluations found.");
            display_help_suggestion("create", true);
            return Ok(());
        }
        
        let use_color = should_use_colors(false); // Would get from global args
        
        if args.detailed {
            display_detailed_list(&evaluations, use_color)?;
        } else {
            display_simple_list(&evaluations, use_color)?;
        }
        
        Ok(())
    }
}

// DELETE COMMAND (aligned with PromptEds delete)
pub mod delete {
    use super::*;
    
    pub async fn execute(args: DeleteArgs) -> Result<()> {
        let storage = Storage::new().await?;
        
        // Check if evaluation exists
        let evaluation = storage.load_evaluation(&args.name).await?
            .ok_or_else(|| EvalError::NotFound(args.name.clone()))?;
        
        // Confirmation (PromptEds pattern)
        if !args.force {
            let action = if args.keep_results {
                "delete configuration for"
            } else {
                "delete"
            };
            
            if !confirm_destructive_action(action, &args.name)? {
                display_info("Deletion cancelled");
                return Ok(());
            }
        }
        
        // Perform deletion
        if args.keep_results {
            // Delete only configuration, keep results
            display_info(&format!("Deleting configuration for '{}'...", args.name));
            // Implementation would delete config only
        } else {
            // Delete everything
            display_info(&format!("Deleting evaluation '{}'...", args.name));
            storage.delete_evaluation(&args.name).await?;
        }
        
        display_success(&format!("Deleted evaluation '{}'", args.name));
        
        Ok(())
    }
}

// EDIT COMMAND (aligned with PromptEds edit)
pub mod edit {
    use super::*;
    
    pub async fn execute(args: EditArgs) -> Result<()> {
        let storage = Storage::new().await?;
        let evaluation = storage.load_evaluation(&args.name).await?
            .ok_or_else(|| EvalError::NotFound(args.name.clone()))?;
        
        if args.config_only {
            // Edit configuration only
            display_info(&format!("Opening configuration editor for '{}'...", args.name));
            // Implementation would open editor
        } else {
            // Interactive editing (PromptEds style)
            return execute_interactive_edit(evaluation, storage).await;
        }
        
        Ok(())
    }
    
    async fn execute_interactive_edit(mut evaluation: Evaluation, storage: Storage) -> Result<()> {
        display_interactive_header(&format!("Editing Evaluation: {}", evaluation.name));
        
        // Edit basic metadata
        let new_description = prompt_text(
            "Description",
            evaluation.description.as_deref(),
            false
        )?;
        
        if !new_description.trim().is_empty() {
            evaluation.description = Some(new_description);
        }
        
        // Edit tags
        let current_tags = evaluation.tags.join(", ");
        let new_tags_str = prompt_text(
            "Tags (comma-separated)",
            if current_tags.is_empty() { None } else { Some(&current_tags) },
            false
        )?;
        
        if !new_tags_str.trim().is_empty() {
            evaluation.tags = new_tags_str.split(',').map(|s| s.trim().to_string()).collect();
        }
        
        // Save changes
        storage.update_evaluation(&evaluation).await?;
        
        display_completion("Updated evaluation", &evaluation.name);
        
        Ok(())
    }
}

// COPY COMMAND (aligned with PromptEds copy/branch)
pub mod copy {
    use super::*;
    
    pub async fn execute(args: CopyArgs) -> Result<()> {
        let storage = Storage::new().await?;
        
        // Load source evaluation
        let source_evaluation = storage.load_evaluation(&args.source).await?
            .ok_or_else(|| EvalError::NotFound(args.source.clone()))?;
        
        // Check if destination already exists
        if storage.evaluation_exists(&args.destination).await? {
            return Err(EvalError::AlreadyExists(args.destination));
        }
        
        // Create copy
        let mut new_evaluation = source_evaluation.clone();
        new_evaluation.id = uuid::Uuid::new_v4().to_string();
        new_evaluation.name = args.destination.clone();
        new_evaluation.created_at = chrono::Utc::now();
        new_evaluation.completed_at = None;
        new_evaluation.status = EvaluationStatus::Configured;
        new_evaluation.results = None; // Don't copy results
        
        storage.save_evaluation(&new_evaluation).await?;
        
        display_success(&format!("Copied '{}' to '{}'", args.source, args.destination));
        display_next_steps(&args.destination);
        
        Ok(())
    }
}