// Interactive prompting style aligned with PromptEds patterns
use dialoguer::{Input, Select, MultiSelect, Confirm, theme::ColorfulTheme};
use console::{style, Term};
use crate::utils::error::Result;
use std::collections::HashMap;

/// PromptEds-style interactive theme with consistent styling
pub fn get_prompteds_theme() -> ColorfulTheme {
    ColorfulTheme {
        defaults_style: console::Style::new().for_stderr().cyan(),
        prompt_style: console::Style::new().for_stderr().bold(),
        prompt_prefix: console::style("?".to_string()).for_stderr().yellow(),
        prompt_suffix: console::style(" ›".to_string()).for_stderr().cyan(),
        success_prefix: console::style("✓".to_string()).for_stderr().green(),
        success_suffix: console::style(" ·".to_string()).for_stderr().cyan(),
        error_prefix: console::style("✗".to_string()).for_stderr().red(),
        error_style: console::Style::new().for_stderr().red(),
        hint_style: console::Style::new().for_stderr().dim(),
        values_style: console::Style::new().for_stderr().cyan(),
        active_item_style: console::Style::new().for_stderr().cyan(),
        inactive_item_style: console::Style::new().for_stderr(),
        active_item_prefix: console::style(" ❯".to_string()).for_stderr().cyan(),
        inactive_item_prefix: console::style("  ".to_string()).for_stderr(),
        checked_item_prefix: console::style(" ✓".to_string()).for_stderr().green(),
        unchecked_item_prefix: console::style(" ○".to_string()).for_stderr().dim(),
        picked_item_prefix: console::style(" ✓".to_string()).for_stderr().green(),
        unpicked_item_prefix: console::style(" ○".to_string()).for_stderr().dim(),
    }
}

/// Display PromptEds-style header for interactive sessions
pub fn display_interactive_header(title: &str) {
    let term = Term::stdout();
    println!("{}", style(format!("🎯 {}", title)).bold().cyan());
    println!();
}

/// Display step header with PromptEds-style formatting
pub fn display_step_header(step: u32, title: &str) {
    println!("{}", style(format!("📝 Step {}: {}", step, title)).bold().blue());
}

/// Display success message with PromptEds-style formatting
pub fn display_success(message: &str) {
    println!("\n✅ {}", style(message).green().bold());
}

/// Display info message with PromptEds-style formatting
pub fn display_info(message: &str) {
    println!("💡 {}", style(message).blue());
}

/// Display warning message with PromptEds-style formatting
pub fn display_warning(message: &str) {
    println!("⚠️  {}", style(message).yellow());
}

/// Display error message with PromptEds-style formatting
pub fn display_error(message: &str) {
    eprintln!("❌ {}", style(message).red());
}

/// Prompt for text input with PromptEds-style formatting
pub fn prompt_text(label: &str, default: Option<&str>, required: bool) -> Result<String> {
    let mut input = Input::with_theme(&get_prompteds_theme())
        .with_prompt(format!("📝 {}", label));
    
    if let Some(default_value) = default {
        input = input.default(default_value.to_string());
    }
    
    if !required {
        input = input.allow_empty(true);
    }
    
    if required {
        input = input.validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.trim().is_empty() {
                Err("This field is required")
            } else {
                Ok(())
            }
        });
    }
    
    Ok(input.interact_text()?)
}

/// Prompt for selection with PromptEds-style formatting
pub fn prompt_select<T: ToString>(label: &str, items: &[T], default: Option<usize>) -> Result<usize> {
    let mut select = Select::with_theme(&get_prompteds_theme())
        .with_prompt(format!("🔍 {}", label))
        .items(items);
    
    if let Some(default_idx) = default {
        select = select.default(default_idx);
    }
    
    Ok(select.interact()?)
}

/// Prompt for multi-selection with PromptEds-style formatting
pub fn prompt_multi_select<T: ToString>(
    label: &str, 
    items: &[T], 
    defaults: Option<&[bool]>
) -> Result<Vec<usize>> {
    let mut multi_select = MultiSelect::with_theme(&get_prompteds_theme())
        .with_prompt(format!("📋 {}", label))
        .items(items);
    
    if let Some(default_selections) = defaults {
        multi_select = multi_select.defaults(default_selections);
    }
    
    Ok(multi_select.interact()?)
}

/// Prompt for confirmation with PromptEds-style formatting
pub fn prompt_confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&get_prompteds_theme())
        .with_prompt(format!("❓ {}", message))
        .default(default)
        .interact()?)
}

/// Display a list of next steps with PromptEds-style formatting
pub fn display_next_steps(name: &str) {
    println!("\nNext steps:");
    println!("  🚀 Run: {}", style(format!("evaleds run {}", name)).cyan());
    println!("  📊 Show: {}", style(format!("evaleds show {}", name)).cyan());
    println!("  ✏️  Edit: {}", style(format!("evaleds edit {}", name)).cyan());
}

/// Display completion message with PromptEds-style formatting
pub fn display_completion(action: &str, name: &str) {
    println!("\n✅ {} '{}' {}", 
        action,
        style(name).cyan().bold(),
        style("successfully").green().bold()
    );
}

/// Display a summary table with PromptEds-style formatting
pub fn display_summary_table(items: &[(String, String)]) {
    println!("\n📋 Configuration Summary:");
    
    let max_key_length = items.iter()
        .map(|(key, _)| key.len())
        .max()
        .unwrap_or(0);
    
    for (key, value) in items {
        println!("  {}: {}", 
            style(format!("{:width$}", key, width = max_key_length)).blue(),
            style(value).cyan()
        );
    }
}

/// Display detected variables with PromptEds-style formatting
pub fn display_detected_variables(variables: &[String]) {
    if !variables.is_empty() {
        println!("🔍 Detected variables: {}", 
            style(variables.join(", ")).yellow().bold()
        );
    }
}

/// Display tags with PromptEds-style formatting
pub fn display_tags(tags: &[String]) {
    if !tags.is_empty() {
        println!("🏷️  Tags: #{}", 
            style(tags.join(" #")).green()
        );
    }
}

/// Display a progress indicator with PromptEds-style formatting
pub fn display_progress(current: usize, total: usize, message: &str) {
    println!("⏳ [{}/{}] {}", 
        style(current).yellow().bold(),
        style(total).yellow().bold(),
        style(message).blue()
    );
}

/// Prompt for variable values with PromptEds-style formatting
pub fn prompt_variables(variables: &[String]) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    
    if !variables.is_empty() {
        println!("🔤 Please provide values for the following variables:");
        
        for variable in variables {
            let value = prompt_text(variable, None, true)?;
            values.insert(variable.clone(), value);
        }
    }
    
    Ok(values)
}

/// Display metadata with PromptEds-style formatting
pub fn display_metadata(name: &str, description: Option<&str>, category: Option<&str>) {
    let mut summary = vec![
        ("Name".to_string(), name.to_string()),
    ];
    
    if let Some(desc) = description {
        summary.push(("Description".to_string(), desc.to_string()));
    }
    
    if let Some(cat) = category {
        summary.push(("Category".to_string(), cat.to_string()));
    }
    
    display_summary_table(&summary);
}

/// Interactive prompt for adding variations
pub fn prompt_add_variations() -> Result<bool> {
    prompt_confirm(
        "Add prompt variations with different variable values?",
        false
    )
}

/// Interactive prompt for variation details
pub fn prompt_variation_details(base_variables: &HashMap<String, String>) -> Result<(String, HashMap<String, String>)> {
    // Get variation name
    let name = prompt_text("Variation name", None, true)?;
    
    let mut variation_vars = base_variables.clone();
    
    println!("Override variables for '{}' (press Enter to keep current value):", 
        style(&name).cyan().bold());
    
    for (key, current_value) in &mut variation_vars {
        let new_value = prompt_text(
            &format!("{} [{}]", key, style(current_value).dim()),
            Some(current_value),
            false
        )?;
        
        if !new_value.trim().is_empty() {
            *current_value = new_value;
        }
    }
    
    Ok((name, variation_vars))
}

/// Display analysis configuration with PromptEds-style formatting
pub fn display_analysis_features() -> Vec<String> {
    vec![
        "Response Metrics (length, readability, sentiment)".to_string(),
        "Similarity Analysis (compare outputs)".to_string(),
        "Content Analysis (keywords, entities, topics)".to_string(),
        "Quality Indicators (relevance, accuracy)".to_string(),
        "Cost Analysis (per provider, per model)".to_string(),
        "Performance Analysis (response times, success rates)".to_string(),
    ]
}

/// Prompt for analysis feature selection
pub fn prompt_analysis_features() -> Result<Vec<usize>> {
    let features = display_analysis_features();
    let defaults = vec![true; features.len()]; // All enabled by default
    
    prompt_multi_select(
        "Select analysis features to enable",
        &features,
        Some(&defaults)
    )
}

/// Display configuration validation results
pub fn display_validation_results(is_valid: bool, errors: &[String], warnings: &[String]) {
    if is_valid {
        display_success("Configuration validated successfully");
    } else {
        display_error("Configuration validation failed");
        for error in errors {
            println!("  ❌ {}", style(error).red());
        }
    }
    
    for warning in warnings {
        println!("  ⚠️  {}", style(warning).yellow());
    }
}

/// Interactive confirmation for destructive actions
pub fn confirm_destructive_action(action: &str, target: &str) -> Result<bool> {
    display_warning(&format!("This will {} '{}'", action, target));
    prompt_confirm("Are you sure you want to continue?", false)
}