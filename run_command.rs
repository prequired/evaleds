// Complete run command implementation
use crate::cli::args::RunArgs;
use crate::core::{evaluation::*, providers::ProviderManager, analysis::AnalysisEngine};
use crate::utils::error::Result;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use futures::stream::{FuturesUnordered, StreamExt};
use std::time::Instant;
use console::style;
use std::collections::HashMap;

pub async fn execute(args: RunArgs) -> Result<()> {
    let storage = crate::core::storage::Storage::new().await?;
    let mut evaluation = storage.load_evaluation(&args.name).await?
        .ok_or_else(|| crate::utils::error::EvalError::NotFound(args.name.clone()))?;
    
    // Check if already completed
    if matches!(evaluation.status, EvaluationStatus::Completed) && !args.force {
        println!("⚠️  Evaluation '{}' already completed. Use --force to re-run.", args.name);
        return Ok(());
    }
    
    println!("🚀 Running evaluation: {}", style(&evaluation.name).cyan().bold());
    
    if let Some(desc) = &evaluation.description {
        println!("📝 {}", desc);
    }
    
    // Update status
    evaluation.status = EvaluationStatus::Running;
    storage.update_evaluation(&evaluation).await?;
    
    // Calculate total executions
    let total_executions = calculate_total_executions(&evaluation.config);
    println!("📊 Total executions planned: {}", style(total_executions).yellow().bold());
    
    // Set up progress tracking
    let multi_progress = MultiProgress::new();
    let overall_progress = multi_progress.add(ProgressBar::new(total_executions as u64));
    overall_progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} executions ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    let provider_manager = ProviderManager::new().await?;
    let mut all_results = Vec::new();
    
    // Execute evaluations
    let start_time = Instant::now();
    
    if evaluation.config.batch_settings.parallel_execution {
        all_results = execute_parallel(&evaluation, &provider_manager, &overall_progress, args.max_concurrent).await?;
    } else {
        all_results = execute_sequential(&evaluation, &provider_manager, &overall_progress).await?;
    }
    
    let execution_time = start_time.elapsed();
    overall_progress.finish_with_message("✅ All executions completed");
    
    println!("⏱️  Total execution time: {:.2}s", execution_time.as_secs_f64());
    
    // Perform analysis
    println!("🔍 Analyzing results...");
    let analysis_progress = ProgressBar::new_spinner();
    analysis_progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    
    let analysis_engine = AnalysisEngine::new();
    
    analysis_progress.set_message("Computing response metrics...");
    let analysis = analysis_engine.analyze_results(&all_results, &evaluation.config.analysis_options).await?;
    
    analysis_progress.set_message("Generating summary...");
    let summary = generate_summary(&all_results, &analysis).await?;
    
    analysis_progress.finish_with_message("✅ Analysis completed");
    
    // Store results
    evaluation.results = Some(EvaluationResults {
        executions: all_results,
        analysis,
        summary,
        report_path: None,
    });
    evaluation.status = EvaluationStatus::Completed;
    evaluation.completed_at = Some(chrono::Utc::now());
    
    storage.update_evaluation(&evaluation).await?;
    
    // Display summary
    display_execution_summary(&evaluation)?;
    
    println!("\n✅ Evaluation '{}' completed successfully", style(&evaluation.name).green().bold());
    println!("📊 View results: {}", style(format!("evaleds view {}", evaluation.name)).cyan());
    
    Ok(())
}

async fn execute_parallel(
    evaluation: &Evaluation,
    provider_manager: &ProviderManager,
    progress: &ProgressBar,
    max_concurrent: Option<u32>,
) -> Result<Vec<ExecutionResult>> {
    let max_concurrent = max_concurrent
        .unwrap_or(evaluation.config.batch_settings.max_concurrent)
        .min(20); // Safety limit
    
    let mut futures = FuturesUnordered::new();
    let mut results = Vec::new();
    let mut pending_executions = Vec::new();
    
    // Prepare all executions
    for prompt_config in &evaluation.config.prompts {
        let resolved_prompts = resolve_prompt_config(prompt_config).await?;
        
        for resolved_prompt in resolved_prompts {
            for provider_config in &evaluation.config.providers {
                for model in &provider_config.models {
                    pending_executions.push((
                        resolved_prompt.clone(),
                        provider_config.clone(),
                        model.clone(),
                    ));
                }
            }
        }
    }
    
    // Execute with concurrency limit
    let mut executing = 0;
    let mut pending_iter = pending_executions.into_iter();
    
    // Start initial batch
    while executing < max_concurrent {
        if let Some((prompt, provider_config, model)) = pending_iter.next() {
            let future = execute_single_prompt(
                provider_manager,
                &provider_config.name,
                &model,
                &prompt,
                &provider_config.settings,
            );
            futures.push(future);
            executing += 1;
        } else {
            break;
        }
    }
    
    // Process results and start new executions
    while !futures.is_empty() {
        if let Some(result) = futures.next().await {
            match result {
                Ok(execution_result) => results.push(execution_result),
                Err(e) => eprintln!("❌ Execution failed: {}", e),
            }
            
            progress.inc(1);
            executing -= 1;
            
            // Start next execution if available
            if let Some((prompt, provider_config, model)) = pending_iter.next() {
                let future = execute_single_prompt(
                    provider_manager,
                    &provider_config.name,
                    &model,
                    &prompt,
                    &provider_config.settings,
                );
                futures.push(future);
                executing += 1;
            }
        }
    }
    
    Ok(results)
}

async fn execute_sequential(
    evaluation: &Evaluation,
    provider_manager: &ProviderManager,
    progress: &ProgressBar,
) -> Result<Vec<ExecutionResult>> {
    let mut results = Vec::new();
    
    for prompt_config in &evaluation.config.prompts {
        let resolved_prompts = resolve_prompt_config(prompt_config).await?;
        
        for resolved_prompt in resolved_prompts {
            for provider_config in &evaluation.config.providers {
                for model in &provider_config.models {
                    match execute_single_prompt(
                        provider_manager,
                        &provider_config.name,
                        model,
                        &resolved_prompt,
                        &provider_config.settings,
                    ).await {
                        Ok(result) => results.push(result),
                        Err(e) => eprintln!("❌ Failed {}/{}: {}", provider_config.name, model, e),
                    }
                    
                    progress.inc(1);
                    
                    // Small delay to be respectful to APIs
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    Ok(results)
}

async fn execute_single_prompt(
    provider_manager: &ProviderManager,
    provider_name: &str,
    model: &str,
    prompt: &str,
    settings: &ModelSettings,
) -> Result<ExecutionResult> {
    provider_manager.execute_prompt(provider_name, model, prompt, settings).await
}

async fn resolve_prompt_config(prompt_config: &PromptConfig) -> Result<Vec<String>> {
    let mut resolved_prompts = Vec::new();
    
    // Get base prompt content
    let base_content = match &prompt_config.source {
        PromptSource::PromptEds { name } => {
            let prompteds = crate::core::prompteds::PromptEdsClient::new().await?;
            prompteds.get_prompt_content(name, &prompt_config.variables).await?
        },
        PromptSource::Direct { content } => content.clone(),
        PromptSource::File { path } => {
            tokio::fs::read_to_string(path).await
                .map_err(|e| crate::utils::error::EvalError::IoError(e))?
        },
    };
    
    // Apply base variables
    let mut rendered_content = base_content;
    for (key, value) in &prompt_config.variables {
        rendered_content = rendered_content.replace(&format!("{{{{{}}}}}", key), value);
    }
    
    resolved_prompts.push(rendered_content.clone());
    
    // Add variations
    for variation in &prompt_config.variations {
        let mut variation_content = base_content.clone();
        for (key, value) in &variation.variables {
            variation_content = variation_content.replace(&format!("{{{{{}}}}}", key), value);
        }
        resolved_prompts.push(variation_content);
    }
    
    Ok(resolved_prompts)
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

async fn generate_summary(
    results: &[ExecutionResult],
    analysis: &AnalysisResults,
) -> Result<ResultSummary> {
    let total_executions = results.len() as u32;
    let successful_executions = results.iter()
        .filter(|r| matches!(r.status, ExecutionStatus::Success))
        .count() as u32;
    let failed_executions = total_executions - successful_executions;
    
    let total_cost = results.iter()
        .map(|r| r.metadata.cost_usd)
        .sum::<f64>();
    
    let avg_response_time = if !results.is_empty() {
        results.iter()
            .map(|r| r.metadata.response_time_ms as f64)
            .sum::<f64>() / results.len() as f64
    } else {
        0.0
    };
    
    let success_rate = if total_executions > 0 {
        (successful_executions as f32 / total_executions as f32) * 100.0
    } else {
        0.0
    };
    
    // Find best performing models
    let best_performing_model = find_best_performing_model(results);
    let most_cost_effective = find_most_cost_effective_model(results);
    let fastest_model = find_fastest_model(results);
    
    Ok(ResultSummary {
        total_executions,
        successful_executions,
        failed_executions,
        total_cost,
        avg_response_time,
        success_rate,
        best_performing_model,
        most_cost_effective,
        fastest_model,
    })
}

fn find_best_performing_model(results: &[ExecutionResult]) -> Option<String> {
    // Simple heuristic: lowest average response time with high success rate
    let mut model_stats: HashMap<String, (f64, u32, u32)> = HashMap::new();
    
    for result in results {
        let model_key = format!("{}/{}", result.provider, result.model);
        let entry = model_stats.entry(model_key).or_insert((0.0, 0, 0));
        entry.0 += result.metadata.response_time_ms as f64;
        entry.1 += 1; // total
        if matches!(result.status, ExecutionStatus::Success) {
            entry.2 += 1; // successes
        }
    }
    
    model_stats.iter()
        .filter(|(_, (_, total, successes))| *successes as f32 / *total as f32 >= 0.8) // 80% success rate
        .min_by(|(_, (time_a, total_a, _)), (_, (time_b, total_b, _))| {
            let avg_a = time_a / *total_a as f64;
            let avg_b = time_b / *total_b as f64;
            avg_a.partial_cmp(&avg_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(model, _)| model.clone())
}

fn find_most_cost_effective_model(results: &[ExecutionResult]) -> Option<String> {
    let mut model_costs: HashMap<String, f64> = HashMap::new();
    
    for result in results {
        if matches!(result.status, ExecutionStatus::Success) {
            let model_key = format!("{}/{}", result.provider, result.model);
            *model_costs.entry(model_key).or_insert(0.0) += result.metadata.cost_usd;
        }
    }
    
    model_costs.iter()
        .min_by(|(_, cost_a), (_, cost_b)| cost_a.partial_cmp(cost_b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(model, _)| model.clone())
}

fn find_fastest_model(results: &[ExecutionResult]) -> Option<String> {
    let mut model_times: HashMap<String, Vec<u64>> = HashMap::new();
    
    for result in results {
        if matches!(result.status, ExecutionStatus::Success) {
            let model_key = format!("{}/{}", result.provider, result.model);
            model_times.entry(model_key).or_insert_with(Vec::new).push(result.metadata.response_time_ms);
        }
    }
    
    model_times.iter()
        .map(|(model, times)| {
            let avg_time = times.iter().sum::<u64>() as f64 / times.len() as f64;
            (model, avg_time)
        })
        .min_by(|(_, time_a), (_, time_b)| time_a.partial_cmp(time_b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(model, _)| model.clone())
}

fn display_execution_summary(evaluation: &Evaluation) -> Result<()> {
    if let Some(results) = &evaluation.results {
        println!("\n📊 Execution Summary:");
        println!("  Total: {}", style(results.summary.total_executions).yellow().bold());
        println!("  Success: {}", style(results.summary.successful_executions).green().bold());
        println!("  Failed: {}", style(results.summary.failed_executions).red().bold());
        println!("  Success Rate: {:.1}%", style(results.summary.success_rate).cyan().bold());
        println!("  Total Cost: ${:.4}", style(results.summary.total_cost).magenta().bold());
        println!("  Avg Response Time: {:.0}ms", style(results.summary.avg_response_time).blue().bold());
        
        if let Some(best) = &results.summary.best_performing_model {
            println!("  Best Performing: {}", style(best).green().bold());
        }
        if let Some(cheapest) = &results.summary.most_cost_effective {
            println!("  Most Cost Effective: {}", style(cheapest).green().bold());
        }
        if let Some(fastest) = &results.summary.fastest_model {
            println!("  Fastest: {}", style(fastest).green().bold());
        }
    }
    
    Ok(())
}