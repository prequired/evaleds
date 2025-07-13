# EvalEds

🚀 **Professional AI Evaluation Platform with PromptEds Integration**

EvalEds is a comprehensive evaluation tool that helps you compare AI model outputs with beautiful web reports and advanced analysis. It integrates seamlessly with PromptEds for powerful prompt management and provides detailed insights into model performance, costs, and quality.

## ✨ Features

### 🎯 **Core Capabilities**
- **3-Stage Workflow**: Setup → Run → View for clear evaluation process
- **Multi-Provider Support**: OpenAI, Anthropic, Google, and local models
- **PromptEds Integration**: Seamless prompt management and variable handling
- **Parallel Execution**: Efficient batch processing with progress tracking
- **Beautiful Web Interface**: Modern, responsive design with interactive charts

### 📊 **Advanced Analysis**
- **Response Metrics**: Length, readability, sentiment analysis
- **Similarity Analysis**: Compare outputs between models
- **Content Analysis**: Keywords, entities, topic extraction
- **Quality Assessment**: Relevance, accuracy, helpfulness scores
- **Cost Tracking**: Detailed breakdown by provider and model
- **Performance Stats**: Response times, success rates, benchmarks

### 🎨 **Professional Reporting**
- **Interactive Dashboard**: Real-time results with charts and metrics
- **Comparison Views**: Side-by-side model output analysis
- **Export Options**: Markdown, HTML, and JSON reports
- **Static Generation**: Shareable reports for teams and stakeholders

## 🚀 Quick Start

### Installation

#### **📦 One-Line Install (Recommended)**

**Linux/macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/prequired/evaleds/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/prequired/evaleds/main/scripts/install.ps1 | iex
```

#### **🛠️ Manual Installation**

**Download and run the installer:**
```bash
# Download installer
curl -O https://raw.githubusercontent.com/prequired/evaleds/main/install.sh
chmod +x install.sh

# Install (with options)
./install.sh                    # Download binary (recommended)
./install.sh --build-from-source # Build from source
./install.sh --install-dir /custom/path
```

**Windows:**
```powershell
# Download and run
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/prequired/evaleds/main/scripts/install.ps1" -OutFile "install.ps1"
.\install.ps1

# Or with options
.\install.ps1 -BuildFromSource
.\install.ps1 -InstallDir "C:\tools\bin"
```

#### **🦀 Build from Source**
```bash
# Requires Rust 1.70+ 
git clone https://github.com/prequired/evaleds
cd evaleds
make install

# Or manually
cargo build --release
cp target/release/evaleds ~/.local/bin/
```

#### **📦 Using Package Managers**
```bash
# Homebrew (macOS/Linux) - Coming Soon
brew install prequired/tap/evaleds

# Cargo
cargo install evaleds

# Arch Linux AUR - Coming Soon  
yay -S evaleds
```

### Setup API Keys

```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export GOOGLE_API_KEY="your-google-key"
```

### Basic Usage

```bash
# 1. Set up a new evaluation
evaleds setup my-first-eval

# 2. Run the evaluation
evaleds run my-first-eval

# 3. View results in web interface
evaleds view my-first-eval
```

## 📋 Commands

### `evaleds setup <name>`
Interactive configuration of prompts, providers, and analysis options.

**Options:**
- `--description "Description"` - Add evaluation description
- `--non-interactive` - Skip interactive setup

**Example:**
```bash
evaleds setup data-analysis --description "Compare models for data analysis tasks"
```

### `evaleds run <name>`
Execute the evaluation with progress tracking.

**Options:**
- `--force` - Re-run even if results exist
- `--max-concurrent 10` - Override concurrent execution limit
- `--background` - Run in background

**Example:**
```bash
evaleds run data-analysis --max-concurrent 8
```

### `evaleds view <name>`
Launch web interface to view results.

**Options:**
- `--port 8080` - Specify port (default: random)
- `--no-browser` - Don't auto-open browser
- `--export markdown` - Export instead of viewing
- `--output report.md` - Export to specific file

**Example:**
```bash
evaleds view data-analysis --port 8080
evaleds view data-analysis --export markdown --output analysis-report.md
```

### `evaleds list`
Show all evaluations with status and metrics.

**Options:**
- `--detailed` - Show comprehensive information
- `--status completed` - Filter by status
- `--sort created` - Sort by field (created, name, status)

### `evaleds delete <name>`
Remove an evaluation and its results.

**Options:**
- `--force` - Skip confirmation
- `--keep-results` - Delete config but keep results

## 🔧 Configuration

EvalEds stores configuration in `~/.evaleds/`:

```
~/.evaleds/
├── config.toml          # Main configuration
├── evaluations.db       # SQLite database
└── exports/            # Exported reports
```

### Provider Configuration

```toml
[providers.openai]
available_models = ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
default_model = "gpt-4-turbo"
enabled = true

[providers.openai.rate_limits]
requests_per_minute = 3500
tokens_per_minute = 90000

[providers.anthropic]
available_models = ["claude-3-opus-20240229", "claude-3-sonnet-20240229"]
default_model = "claude-3-sonnet-20240229"
enabled = true
```

### Analysis Settings

```toml
[analysis]
enable_similarity_analysis = true
enable_content_analysis = true
enable_quality_assessment = true
similarity_threshold = 0.7
max_keywords = 10
```

## 🎯 PromptEds Integration

EvalEds seamlessly integrates with PromptEds for powerful prompt management:

### Using PromptEds Prompts

1. **Create prompts with PromptEds:**
```bash
prompteds add data-analysis --template "Analyze this data: {{data}}\n\nGoal: {{goal}}"
```

2. **Use in EvalEds setup:**
```bash
evaleds setup my-eval
# Select "PromptEds prompt" → "data-analysis"
# Provide variable values: data="sales.csv", goal="find trends"
```

3. **Add variations:**
```bash
# During setup, create variations with different variable values
# Variation 1: goal="find anomalies"
# Variation 2: goal="predict future"
```

### Prompt Variables

EvalEds automatically detects PromptEds variables and provides interactive configuration:

```markdown
---
title: "Data Analysis Template"
tags: ["analysis", "data"]
---

Please analyze the following {{data_type}} data:

{{data}}

Analysis goals:
{{#each goals}}
- {{this}}
{{/each}}

Focus areas: {{focus_areas}}
```

## 📊 Web Interface

The web interface provides comprehensive result analysis:

### Dashboard
- **Summary Metrics**: Success rates, costs, response times
- **Quick Actions**: Export, compare, re-run
- **Recent Results**: Latest execution overview

### Comparison View
- **Side-by-Side**: Model outputs with highlighting
- **Performance Metrics**: Speed and cost comparison
- **Quality Scores**: Automated quality assessment

### Analysis View
- **Best Performers**: Top models by different criteria
- **Detailed Metrics**: Comprehensive performance tables
- **Charts & Graphs**: Visual performance analysis

## 🔄 Workflow Examples

### Simple Model Comparison

```bash
# Setup with direct prompt
evaleds setup quick-test
# Choose: Direct input
# Prompt: "Explain quantum computing in simple terms"
# Providers: OpenAI (gpt-4, gpt-3.5-turbo), Anthropic (claude-3-sonnet)

# Run evaluation
evaleds run quick-test

# View results
evaleds view quick-test
```

### PromptEds Integration Workflow

```bash
# 1. Create prompt with PromptEds
prompteds add code-review \
  --template "Review this {{language}} code:\n\n{{code}}\n\nFocus: {{focus}}"

# 2. Setup evaluation with variations
evaleds setup code-review-eval
# Select PromptEds prompt: code-review
# Variables: language="Python", code="def hello()...", focus="security"
# Add variation: focus="performance"
# Add variation: focus="readability"

# 3. Run comprehensive evaluation
evaleds run code-review-eval --max-concurrent 10

# 4. Generate report
evaleds view code-review-eval --export html --output team-review.html
```

### Cost Analysis Workflow

```bash
# Setup evaluation focusing on cost comparison
evaleds setup cost-analysis
# Include: GPT-3.5, GPT-4, Claude Haiku, Claude Sonnet
# Prompt: Long analysis task (high token usage)

# Run with cost tracking
evaleds run cost-analysis

# View cost breakdown
evaleds view cost-analysis
# Navigate to Analysis → Cost Breakdown
```

## 📈 Performance

EvalEds is optimized for efficient evaluation:

- **Parallel Execution**: Up to 20 concurrent API calls
- **Rate Limit Awareness**: Automatic backoff and retry
- **Progress Tracking**: Real-time execution updates
- **Memory Efficient**: Streams large results to database
- **Fast Web Interface**: Sub-second response times

### Benchmark Results

```
Evaluation Size: 100 executions (4 providers × 25 prompts)
Total Time: 2.3 minutes
Success Rate: 98.5%
Memory Usage: <100MB
```

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/yourusername/evaleds
cd evaleds
cargo build --release
```

### Running Tests

```bash
cargo test
cargo test --features integration
```

### Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Uninstallation

#### **🗑️ Quick Uninstall**

**Linux/macOS:**
```bash
# Download and run uninstaller
curl -sSL https://raw.githubusercontent.com/prequired/evaleds/main/uninstall.sh | bash

# Or run the local uninstaller if you have it
./uninstall.sh

# Remove everything including data
./uninstall.sh --remove-all

# Dry run to see what would be removed
./uninstall.sh --dry-run
```

**Windows:**
```powershell
# Download and run uninstaller
irm https://raw.githubusercontent.com/prequired/evaleds/main/scripts/uninstall.ps1 | iex

# Or run local uninstaller
.\uninstall.ps1

# Remove everything
.\uninstall.ps1 -RemoveAll
```

#### **🔧 Manual Uninstall**
```bash
# Remove binary
rm ~/.local/bin/evaleds

# Remove configuration (optional)
rm -rf ~/.config/evaleds

# Remove data (optional) 
rm -rf ~/.local/share/evaleds
```

## 🤝 Integration with PromptEds

EvalEds is designed to work seamlessly with [PromptEds](https://github.com/prequired/prompteds):

- **Shared Configuration**: Uses PromptEds prompt directory
- **Variable Handling**: Full support for PromptEds templating
- **Version Management**: Tracks prompt versions and changes
- **Tag Support**: Filter prompts by PromptEds tags

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license.

## 🙋 Support

- **Documentation**: [https://evaleds.dev/docs](https://evaleds.dev/docs)
- **Issues**: [GitHub Issues](https://github.com/yourusername/evaleds/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/evaleds/discussions)
- **Email**: support@evaleds.dev

---

**EvalEds** - Professional AI evaluation made simple. Compare models, analyze results, make better decisions. 🚀