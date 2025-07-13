// Updated CLI argument structure aligned with PromptEds patterns
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(
    name = "evaleds",
    version = env!("CARGO_PKG_VERSION"),
    author = "EvalEds Team",
    about = "AI evaluation platform with PromptEds integration",
    long_about = "EvalEds helps you compare AI model outputs with comprehensive analysis and beautiful reports."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Use porcelain (machine-readable) output format
    #[arg(long, global = true)]
    pub porcelain: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new evaluation configuration
    #[command(
        about = "Create a new evaluation configuration",
        long_about = "Create a new evaluation configuration with prompts, providers, and analysis options. Use --interactive for guided setup.",
        after_help = "EXAMPLES:\n    evaleds create model-comparison --description \"Compare GPT vs Claude\"\n    evaleds create quick-test --tag benchmark --category performance\n    evaleds create comprehensive --interactive"
    )]
    Create(CreateArgs),
    
    /// Run an evaluation
    #[command(
        about = "Run an evaluation",
        long_about = "Execute an evaluation with progress tracking and parallel execution support.",
        after_help = "EXAMPLES:\n    evaleds run model-comparison\n    evaleds run quick-test --force\n    evaleds run comprehensive --max-concurrent 10"
    )]
    Run(RunArgs),
    
    /// Show evaluation results
    #[command(
        about = "Show evaluation results",
        long_about = "Show evaluation results with optional web interface or export options.",
        after_help = "EXAMPLES:\n    evaleds show model-comparison          # Show results summary\n    evaleds show model-comparison --web    # Launch web interface\n    evaleds show model-comparison --export markdown -o report.md"
    )]
    Show(ShowArgs),
    
    /// List evaluations
    #[command(
        about = "List evaluations",
        long_about = "List evaluations with filtering and sorting options.",
        after_help = "EXAMPLES:\n    evaleds list                           # List all evaluations\n    evaleds list --tag benchmark --detailed\n    evaleds list --status completed --sort created"
    )]
    List(ListArgs),
    
    /// Delete an evaluation
    #[command(
        about = "Delete an evaluation",
        long_about = "Delete an evaluation and optionally keep results.",
        after_help = "EXAMPLES:\n    evaleds delete old-test\n    evaleds delete failed-run --force\n    evaleds delete outdated --keep-results"
    )]
    Delete(DeleteArgs),
    
    /// Edit evaluation configuration
    #[command(
        about = "Edit evaluation configuration",
        long_about = "Edit an existing evaluation configuration.",
        after_help = "EXAMPLES:\n    evaleds edit model-comparison\n    evaleds edit quick-test --config-only"
    )]
    Edit(EditArgs),
    
    /// Copy evaluation configuration
    #[command(
        about = "Copy evaluation configuration",
        long_about = "Copy an evaluation configuration to create a new evaluation.",
        after_help = "EXAMPLES:\n    evaleds copy model-comparison extended-comparison\n    evaleds copy baseline experiment-v2"
    )]
    Copy(CopyArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// Name of the evaluation
    pub name: String,
    
    /// Description of the evaluation
    #[arg(short, long)]
    pub description: Option<String>,
    
    /// Add tag (can be used multiple times)
    #[arg(short = 't', long = "tag", action = clap::ArgAction::Append)]
    pub tags: Vec<String>,
    
    /// Category for organization
    #[arg(short, long)]
    pub category: Option<String>,
    
    /// Use interactive creation wizard
    #[arg(short, long)]
    pub interactive: bool,
    
    /// Configuration from file
    #[arg(short, long)]
    pub file: Option<String>,
}

#[derive(Args)]
pub struct RunArgs {
    /// Name of the evaluation to run
    pub name: String,
    
    /// Force re-run even if results exist
    #[arg(short, long)]
    pub force: bool,
    
    /// Run in background
    #[arg(long)]
    pub background: bool,
    
    /// Maximum concurrent executions
    #[arg(long)]
    pub max_concurrent: Option<u32>,
    
    /// Dry run - show what would be executed
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Name of the evaluation to show
    pub name: String,
    
    /// Show raw configuration
    #[arg(long)]
    pub raw: bool,
    
    /// Show metadata only
    #[arg(long)]
    pub metadata: bool,
    
    /// Launch web interface
    #[arg(short, long)]
    pub web: bool,
    
    /// Export format (markdown, html, json)
    #[arg(long)]
    pub export: Option<String>,
    
    /// Export output file
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter by tag
    #[arg(short = 't', long = "tag", action = clap::ArgAction::Append)]
    pub tags: Vec<String>,
    
    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,
    
    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,
    
    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,
    
    /// Sort by field (name, created, updated, status)
    #[arg(short, long, default_value = "name")]
    pub sort: String,
    
    /// Reverse sort order
    #[arg(short, long)]
    pub reverse: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Name of the evaluation to delete
    pub name: String,
    
    /// Force deletion without confirmation
    #[arg(short, long)]
    pub force: bool,
    
    /// Keep results but delete configuration
    #[arg(long)]
    pub keep_results: bool,
}

#[derive(Args)]
pub struct EditArgs {
    /// Name of the evaluation to edit
    pub name: String,
    
    /// Edit configuration only
    #[arg(long)]
    pub config_only: bool,
}

#[derive(Args)]
pub struct CopyArgs {
    /// Source evaluation name
    pub source: String,
    
    /// Destination evaluation name
    pub destination: String,
}