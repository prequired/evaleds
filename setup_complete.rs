// Complete setup command implementation
// This completes the setup.rs file that was cut off

use crate::cli::args::SetupArgs;
use crate::core::{evaluation::*, prompteds::PromptEdsClient};
use crate::utils::config::load_provider_configs;
use crate::utils::error::Result;
use dialoguer::{Input, Select, MultiSelect, Confirm, theme::ColorfulTheme};
use console::style;
use std::collections::HashMap;

async fn setup_analysis_options() -> Result<AnalysisOptions> {
    println!("Select analysis features to enable:");
    
    let features = vec![
        "Response Metrics (length, readability, sentiment)",
        "Similarity Analysis (compare outputs)",
        "Content Analysis (keywords, entities, topics)",
        "Quality Indicators (relevance, accuracy)",
        "Cost Analysis (per provider, per model)",
        "Performance Analysis (response times, success rates)",
    ];
    
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Analysis features")
        .items(&features)
        .defaults(&[true, true, true, true, true, true])
        .interact()?;
    
    Ok(AnalysisOptions {
        response_metrics: selections.contains(&0),
        similarity_analysis: selections.contains(&1),
        content_analysis: selections.contains(&2),
        quality_indicators: selections.contains(&3),
        cost_analysis: selections.contains(&4),
        performance_analysis: selections.contains(&5),
    })
}

async fn setup_batch_settings() -> Result<BatchSettings> {
    let parallel_execution = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable parallel execution?")
        .default(true)
        .interact()?;
    
    let max_concurrent: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Maximum concurrent requests")
        .default("5".to_string())
        .validate_with(|input: &String| -> Result<(), &str> {
            match input.parse::<u32>() {
                Ok(n) if n > 0 && n <= 20 => Ok(()),
                _ => Err("Must be a number between 1 and 20"),
            }
        })
        .interact_text()?;
    
    let retry_attempts: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Retry attempts for failed requests")
        .default("3".to_string())
        .interact_text()?;
    
    let timeout_seconds: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Timeout per request (seconds)")
        .default("120".to_string())
        .interact_text()?;
    
    Ok(BatchSettings {
        parallel_execution,
        max_concurrent: max_concurrent.parse().unwrap_or(5),
        retry_attempts: retry_attempts.parse().unwrap_or(3),
        timeout_seconds: timeout_seconds.parse().unwrap_or(120),
    })
}

async fn setup_non_interactive(args: SetupArgs) -> Result<()> {
    // Basic non-interactive setup with defaults
    let config = EvaluationConfig {
        prompts: Vec::new(),
        providers: Vec::new(),
        analysis_options: AnalysisOptions::default(),
        batch_settings: BatchSettings::default(),
    };
    
    let evaluation = Evaluation::new(args.name.clone(), config);
    let storage = crate::core::storage::Storage::new().await?;
    storage.save_evaluation(&evaluation).await?;
    
    println!("✅ Created basic evaluation '{}'. Use 'evaleds setup {}' to configure.", 
        args.name, args.name);
    
    Ok(())
}