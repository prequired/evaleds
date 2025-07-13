// Output formatting and color scheme aligned with PromptEds
use console::{style, Term, Color};
use crate::core::evaluation::{Evaluation, EvaluationStatus, EvaluationSummary};
use crate::core::storage::StorageStats;
use crate::utils::error::Result;
use chrono::{DateTime, Utc};

/// PromptEds-style color scheme
pub struct PromptEdsColors;

impl PromptEdsColors {
    // Primary colors matching PromptEds
    pub fn name() -> console::Style { style("").cyan().bold() }
    pub fn version() -> console::Style { style("").yellow().bold() }
    pub fn success() -> console::Style { style("").green().bold() }
    pub fn error() -> console::Style { style("").red().bold() }
    pub fn warning() -> console::Style { style("").yellow() }
    pub fn info() -> console::Style { style("").blue() }
    pub fn dim() -> console::Style { style("").dim() }
    pub fn bold() -> console::Style { style("").bold() }
    
    // Status colors
    pub fn status_completed() -> console::Style { style("").green() }
    pub fn status_running() -> console::Style { style("").yellow() }
    pub fn status_failed() -> console::Style { style("").red() }
    pub fn status_configured() -> console::Style { style("").blue() }
    
    // Accent colors
    pub fn tag() -> console::Style { style("").green() }
    pub fn category() -> console::Style { style("").blue() }
    pub fn cost() -> console::Style { style("").magenta() }
    pub fn metric() -> console::Style { style("").cyan() }
}

/// Check if colors should be used based on terminal capabilities and user preferences
pub fn should_use_colors(no_color: bool) -> bool {
    if no_color {
        return false;
    }
    
    let term = Term::stdout();
    term.features().colors_supported()
}

/// Format evaluation name with PromptEds styling
pub fn format_evaluation_name(name: &str, use_color: bool) -> String {
    if use_color {
        PromptEdsColors::name().apply_to(name).to_string()
    } else {
        name.to_string()
    }
}

/// Format evaluation status with PromptEds styling
pub fn format_evaluation_status(status: &EvaluationStatus, use_color: bool) -> String {
    let (icon, text, style_fn): (&str, &str, fn() -> console::Style) = match status {
        EvaluationStatus::Completed => ("✅", "completed", PromptEdsColors::status_completed),
        EvaluationStatus::Running => ("🔄", "running", PromptEdsColors::status_running),
        EvaluationStatus::Failed => ("❌", "failed", PromptEdsColors::status_failed),
        EvaluationStatus::Configured => ("⚙️", "configured", PromptEdsColors::status_configured),
    };
    
    if use_color {
        format!("{} {}", icon, style_fn().apply_to(text))
    } else {
        format!("{} {}", icon, text)
    }
}

/// Format tags with PromptEds styling
pub fn format_tags(tags: &[String], use_color: bool) -> String {
    if tags.is_empty() {
        return String::new();
    }
    
    let tag_string = format!("#{}", tags.join(" #"));
    
    if use_color {
        format!(" {}", PromptEdsColors::tag().apply_to(&tag_string))
    } else {
        format!(" {}", tag_string)
    }
}

/// Format cost with PromptEds styling
pub fn format_cost(cost: f64, use_color: bool) -> String {
    let cost_str = format!("${:.4}", cost);
    
    if use_color {
        PromptEdsColors::cost().apply_to(&cost_str).to_string()
    } else {
        cost_str
    }
}

/// Format metric value with PromptEds styling
pub fn format_metric(value: &str, use_color: bool) -> String {
    if use_color {
        PromptEdsColors::metric().apply_to(value).to_string()
    } else {
        value.to_string()
    }
}

/// Format timestamp in PromptEds style
pub fn format_timestamp(timestamp: &DateTime<Utc>, use_color: bool) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);
    
    let time_str = if duration.num_days() > 7 {
        timestamp.format("%Y-%m-%d").to_string()
    } else if duration.num_days() > 0 {
        format!("{} day{} ago", duration.num_days(), if duration.num_days() == 1 { "" } else { "s" })
    } else if duration.num_hours() > 0 {
        format!("{} hour{} ago", duration.num_hours(), if duration.num_hours() == 1 { "" } else { "s" })
    } else if duration.num_minutes() > 0 {
        format!("{} minute{} ago", duration.num_minutes(), if duration.num_minutes() == 1 { "" } else { "s" })
    } else {
        "just now".to_string()
    };
    
    if use_color {
        PromptEdsColors::dim().apply_to(&time_str).to_string()
    } else {
        time_str
    }
}

/// Display simple evaluation list (PromptEds style)
pub fn display_simple_list(evaluations: &[EvaluationSummary], use_color: bool) -> Result<()> {
    for evaluation in evaluations {
        print!("{}", format_evaluation_name(&evaluation.name, use_color));
        print!(" {}", format_evaluation_status(&evaluation.status, use_color));
        
        // Show execution count if available
        if evaluation.execution_count > 0 {
            let count_str = format!("[{} executions]", evaluation.execution_count);
            if use_color {
                print!(" {}", PromptEdsColors::info().apply_to(&count_str));
            } else {
                print!(" {}", count_str);
            }
        }
        
        // Show description if available
        if let Some(description) = &evaluation.description {
            if use_color {
                print!(" {}", PromptEdsColors::dim().apply_to(&format!("- {}", description)));
            } else {
                print!(" - {}", description);
            }
        }
        
        // Show timestamp
        print!(" {}", format_timestamp(&evaluation.created_at, use_color));
        
        println!();
    }
    
    // Summary line
    let count_str = format!("{} evaluation{} found", 
        evaluations.len(), 
        if evaluations.len() == 1 { "" } else { "s" }
    );
    
    if use_color {
        println!("\n{}", PromptEdsColors::bold().apply_to(&count_str));
    } else {
        println!("\n{}", count_str);
    }
    
    Ok(())
}

/// Display detailed evaluation list (PromptEds style)
pub fn display_detailed_list(evaluations: &[EvaluationSummary], use_color: bool) -> Result<()> {
    for (i, evaluation) in evaluations.iter().enumerate() {
        if i > 0 {
            println!();
        }
        
        // Header line
        println!("{} {}", 
            format_evaluation_name(&evaluation.name, use_color),
            format_evaluation_status(&evaluation.status, use_color)
        );
        
        // Description
        if let Some(description) = &evaluation.description {
            if use_color {
                println!("  📝 {}", PromptEdsColors::dim().apply_to(description));
            } else {
                println!("  Description: {}", description);
            }
        }
        
        // Execution count and timestamps
        if use_color {
            println!("  📊 {} executions", PromptEdsColors::metric().apply_to(&evaluation.execution_count.to_string()));
            println!("  📅 Created {}", format_timestamp(&evaluation.created_at, use_color));
            
            if let Some(completed_at) = &evaluation.completed_at {
                println!("  ✅ Completed {}", format_timestamp(completed_at, use_color));
            }
        } else {
            println!("  Executions: {}", evaluation.execution_count);
            println!("  Created: {}", format_timestamp(&evaluation.created_at, use_color));
            
            if let Some(completed_at) = &evaluation.completed_at {
                println!("  Completed: {}", format_timestamp(completed_at, use_color));
            }
        }
    }
    
    // Summary line
    let count_str = format!("{} evaluation{} found", 
        evaluations.len(), 
        if evaluations.len() == 1 { "" } else { "s" }
    );
    
    if use_color {
        println!("\n{}", PromptEdsColors::bold().apply_to(&count_str));
    } else {
        println!("\n{}", count_str);
    }
    
    Ok(())
}

/// Display storage statistics (PromptEds style)
pub fn display_storage_stats(stats: &StorageStats, use_color: bool) -> Result<()> {
    println!("📊 EvalEds Statistics");
    println!();
    
    let stats_data = vec![
        ("Total Evaluations", stats.total_evaluations.to_string()),
        ("Completed", stats.completed_evaluations.to_string()),
        ("Running", stats.running_evaluations.to_string()),
        ("Total Executions", stats.total_executions.to_string()),
        ("Total Cost", format!("${:.4}", stats.total_cost)),
    ];
    
    let max_label_width = stats_data.iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    
    for (label, value) in stats_data {
        if use_color {
            println!("  {}: {}", 
                PromptEdsColors::info().apply_to(&format!("{:width$}", label, width = max_label_width)),
                PromptEdsColors::metric().apply_to(&value)
            );
        } else {
            println!("  {}: {}", 
                format!("{:width$}", label, width = max_label_width),
                value
            );
        }
    }
    
    Ok(())
}

/// Display evaluation metadata (PromptEds style)
pub fn display_evaluation_metadata(evaluation: &Evaluation, use_color: bool) -> Result<()> {
    println!("{}", format_evaluation_name(&evaluation.name, use_color));
    
    if let Some(description) = &evaluation.description {
        if use_color {
            println!("📝 {}", PromptEdsColors::dim().apply_to(description));
        } else {
            println!("Description: {}", description);
        }
    }
    
    println!();
    
    // Configuration summary
    let config_data = vec![
        ("Status", format!("{:?}", evaluation.status)),
        ("Prompts", evaluation.config.prompts.len().to_string()),
        ("Providers", evaluation.config.providers.len().to_string()),
        ("Created", format_timestamp(&evaluation.created_at, use_color)),
    ];
    
    if let Some(completed_at) = &evaluation.completed_at {
        let mut config_data = config_data;
        config_data.push(("Completed", format_timestamp(completed_at, use_color)));
    }
    
    let max_label_width = config_data.iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    
    for (label, value) in config_data {
        if use_color {
            println!("  {}: {}", 
                PromptEdsColors::info().apply_to(&format!("{:width$}", label, width = max_label_width)),
                value
            );
        } else {
            println!("  {}: {}", 
                format!("{:width$}", label, width = max_label_width),
                value
            );
        }
    }
    
    Ok(())
}

/// Display evaluation results summary (PromptEds style)
pub fn display_results_summary(evaluation: &Evaluation, use_color: bool) -> Result<()> {
    if let Some(results) = &evaluation.results {
        println!();
        if use_color {
            println!("{}", PromptEdsColors::bold().apply_to("📊 Results Summary"));
        } else {
            println!("Results Summary");
        }
        
        let summary_data = vec![
            ("Total Executions", results.summary.total_executions.to_string()),
            ("Successful", results.summary.successful_executions.to_string()),
            ("Failed", results.summary.failed_executions.to_string()),
            ("Success Rate", format!("{:.1}%", results.summary.success_rate)),
            ("Total Cost", format!("${:.4}", results.summary.total_cost)),
            ("Avg Response Time", format!("{:.0}ms", results.summary.avg_response_time)),
        ];
        
        let max_label_width = summary_data.iter()
            .map(|(label, _)| label.len())
            .max()
            .unwrap_or(0);
        
        for (label, value) in summary_data {
            if use_color {
                println!("  {}: {}", 
                    PromptEdsColors::info().apply_to(&format!("{:width$}", label, width = max_label_width)),
                    PromptEdsColors::metric().apply_to(&value)
                );
            } else {
                println!("  {}: {}", 
                    format!("{:width$}", label, width = max_label_width),
                    value
                );
            }
        }
        
        // Best performers
        if let Some(best) = &results.summary.best_performing_model {
            if use_color {
                println!("  {}: {}", 
                    PromptEdsColors::info().apply_to(&format!("{:width$}", "Best Performing", width = max_label_width)),
                    PromptEdsColors::success().apply_to(best)
                );
            } else {
                println!("  {}: {}", 
                    format!("{:width$}", "Best Performing", width = max_label_width),
                    best
                );
            }
        }
        
        if let Some(cheapest) = &results.summary.most_cost_effective {
            if use_color {
                println!("  {}: {}", 
                    PromptEdsColors::info().apply_to(&format!("{:width$}", "Most Cost Effective", width = max_label_width)),
                    PromptEdsColors::success().apply_to(cheapest)
                );
            } else {
                println!("  {}: {}", 
                    format!("{:width$}", "Most Cost Effective", width = max_label_width),
                    cheapest
                );
            }
        }
        
        if let Some(fastest) = &results.summary.fastest_model {
            if use_color {
                println!("  {}: {}", 
                    PromptEdsColors::info().apply_to(&format!("{:width$}", "Fastest", width = max_label_width)),
                    PromptEdsColors::success().apply_to(fastest)
                );
            } else {
                println!("  {}: {}", 
                    format!("{:width$}", "Fastest", width = max_label_width),
                    fastest
                );
            }
        }
    }
    
    Ok(())
}

/// Display porcelain (machine-readable) output
pub fn display_porcelain_list(evaluations: &[EvaluationSummary]) -> Result<()> {
    for evaluation in evaluations {
        println!("{}|{:?}|{}|{}", 
            evaluation.name,
            evaluation.status,
            evaluation.execution_count,
            evaluation.created_at.to_rfc3339()
        );
    }
    
    Ok(())
}

/// Display help suggestions (PromptEds style)
pub fn display_help_suggestion(command: &str, use_color: bool) {
    if use_color {
        println!("💡 For help, run: {}", 
            PromptEdsColors::info().apply_to(&format!("evaleds {} --help", command))
        );
    } else {
        println!("For help, run: evaleds {} --help", command);
    }
}