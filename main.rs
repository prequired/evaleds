// Main CLI entry point for EvalEds - aligned with PromptEds patterns
use clap::Parser;

mod cli;
mod core;
mod utils;
mod ui;
mod web;

use cli::args::Cli;
use utils::error::EvalError;

#[tokio::main]
async fn main() {
    // Initialize logging based on environment
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Set up panic hook for better error reporting (PromptEds style)
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("evaleds: fatal: {}", panic_info);
        eprintln!("Please report this issue at: https://github.com/yourusername/evaleds/issues");
        std::process::exit(128);
    }));
    
    // Execute the appropriate command
    let result = match cli.command {
        cli::args::Commands::Create(args) => {
            cli::commands::create::execute(args).await
        },
        cli::args::Commands::Run(args) => {
            cli::commands::run::execute(args).await
        },
        cli::args::Commands::Show(args) => {
            cli::commands::show::execute(args).await
        },
        cli::args::Commands::List(args) => {
            cli::commands::list::execute(args).await
        },
        cli::args::Commands::Delete(args) => {
            cli::commands::delete::execute(args).await
        },
        cli::args::Commands::Edit(args) => {
            cli::commands::edit::execute(args).await
        },
        cli::args::Commands::Copy(args) => {
            cli::commands::copy::execute(args).await
        },
    };
    
    // Handle errors with GNU-style formatting (PromptEds pattern)
    if let Err(error) = result {
        let command = std::env::args().nth(1).unwrap_or_else(|| "evaleds".to_string());
        eprintln!("{}", error.format_error(&command));
        
        // Show helpful suggestions if available
        if let Some(suggestion) = error.get_suggestion() {
            eprintln!("\n{}", suggestion);
        }
        
        std::process::exit(error.exit_code());
    }
}

// CLI commands module structure (PromptEds aligned)
mod cli {
    pub mod args {
        pub use crate::cli_args::*;
    }
    
    pub mod commands {
        pub use crate::commands_aligned::*;
    }
}

// Core functionality modules
mod core {
    pub mod evaluation {
        // Re-export from lib
        pub use evaleds::core::evaluation::*;
    }
    
    pub mod storage {
        pub use evaleds::core::storage::*;
    }
    
    pub mod providers {
        pub use evaleds::core::providers::*;
    }
    
    pub mod analysis {
        pub use evaleds::core::analysis::*;
    }
}

// Utilities module
mod utils {
    pub mod error {
        pub use evaleds::utils::error::*;
    }
    
    pub mod config {
        pub use evaleds::utils::config::*;
    }
}

// UI module (PromptEds style)
mod ui {
    pub mod interactive_style {
        pub use crate::interactive_style::*;
    }
    
    pub mod output_formatting {
        pub use crate::output_formatting::*;
    }
}

// Web module
mod web {
    pub use evaleds::web::*;
}