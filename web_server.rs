// Web server implementation with templates and handlers
use crate::core::evaluation::Evaluation;
use crate::utils::error::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use handlebars::Handlebars;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub evaluation: Arc<Evaluation>,
    pub handlebars: Arc<Handlebars<'static>>,
}

pub struct WebServer {
    app: Router,
    port: u16,
}

impl WebServer {
    pub async fn new(evaluation: &Evaluation, port: u16) -> Result<Self> {
        // Set up Handlebars
        let mut handlebars = Handlebars::new();
        
        // Register templates
        handlebars.register_template_string("layout", LAYOUT_TEMPLATE)?;
        handlebars.register_template_string("dashboard", DASHBOARD_TEMPLATE)?;
        handlebars.register_template_string("comparison", COMPARISON_TEMPLATE)?;
        handlebars.register_template_string("analysis", ANALYSIS_TEMPLATE)?;
        
        let state = AppState {
            evaluation: Arc::new(evaluation.clone()),
            handlebars: Arc::new(handlebars),
        };
        
        let app = Router::new()
            .route("/", get(dashboard_handler))
            .route("/comparison", get(comparison_handler))
            .route("/analysis", get(analysis_handler))
            .route("/export", get(export_handler))
            .route("/api/results", get(api_results))
            .nest_service("/static", ServeDir::new("static"))
            .with_state(state);
        
        Ok(Self { app, port })
    }
    
    pub async fn start(self) -> Result<String> {
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;
        let addr = listener.local_addr()?;
        let url = format!("http://127.0.0.1:{}", addr.port());
        
        println!("🌐 Web server starting at {}", url);
        
        tokio::spawn(async move {
            axum::serve(listener, self.app).await.unwrap();
        });
        
        Ok(url)
    }
}

async fn dashboard_handler(State(state): State<AppState>) -> impl IntoResponse {
    let evaluation = &state.evaluation;
    
    let data = json!({
        "evaluation": evaluation,
        "title": format!("EvalEds - {}", evaluation.name),
        "page": "dashboard"
    });
    
    match state.handlebars.render("dashboard", &data) {
        Ok(html) => Html(html),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html(format!("<h1>Error rendering template: {}</h1>", e))
        }
    }
}

async fn comparison_handler(State(state): State<AppState>) -> impl IntoResponse {
    let evaluation = &state.evaluation;
    
    let data = json!({
        "evaluation": evaluation,
        "title": format!("Comparison - {}", evaluation.name),
        "page": "comparison"
    });
    
    match state.handlebars.render("comparison", &data) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<h1>Error: {}</h1>", e))
    }
}

async fn analysis_handler(State(state): State<AppState>) -> impl IntoResponse {
    let evaluation = &state.evaluation;
    
    let data = json!({
        "evaluation": evaluation,
        "title": format!("Analysis - {}", evaluation.name),
        "page": "analysis"
    });
    
    match state.handlebars.render("analysis", &data) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<h1>Error: {}</h1>", e))
    }
}

async fn export_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("html");
    
    match format {
        "markdown" => export_markdown(&state.evaluation).await,
        "json" => export_json(&state.evaluation).await,
        _ => export_html(&state.evaluation).await,
    }
}

async fn api_results(State(state): State<AppState>) -> impl IntoResponse {
    axum::Json(&state.evaluation.results)
}

async fn export_markdown(evaluation: &Evaluation) -> Response {
    let markdown = generate_markdown_report(evaluation).await;
    
    axum::response::Response::builder()
        .header("content-type", "text/markdown")
        .header("content-disposition", "attachment; filename=\"report.md\"")
        .body(markdown)
        .unwrap()
}

async fn export_json(evaluation: &Evaluation) -> Response {
    let json = serde_json::to_string_pretty(evaluation).unwrap_or_default();
    
    axum::response::Response::builder()
        .header("content-type", "application/json")
        .header("content-disposition", "attachment; filename=\"evaluation.json\"")
        .body(json)
        .unwrap()
}

async fn export_html(evaluation: &Evaluation) -> Response {
    let html = generate_html_report(evaluation).await;
    
    axum::response::Response::builder()
        .header("content-type", "text/html")
        .header("content-disposition", "attachment; filename=\"report.html\"")
        .body(html)
        .unwrap()
}

async fn generate_markdown_report(evaluation: &Evaluation) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("# Evaluation Report: {}\n\n", evaluation.name));
    
    if let Some(description) = &evaluation.description {
        report.push_str(&format!("{}\n\n", description));
    }
    
    if let Some(results) = &evaluation.results {
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- **Total Executions**: {}\n", results.summary.total_executions));
        report.push_str(&format!("- **Successful**: {}\n", results.summary.successful_executions));
        report.push_str(&format!("- **Failed**: {}\n", results.summary.failed_executions));
        report.push_str(&format!("- **Success Rate**: {:.1}%\n", results.summary.success_rate));
        report.push_str(&format!("- **Total Cost**: ${:.4}\n", results.summary.total_cost));
        report.push_str(&format!("- **Average Response Time**: {:.0}ms\n\n", results.summary.avg_response_time));
        
        if let Some(best) = &results.summary.best_performing_model {
            report.push_str(&format!("- **Best Performing Model**: {}\n", best));
        }
        if let Some(cheapest) = &results.summary.most_cost_effective {
            report.push_str(&format!("- **Most Cost Effective**: {}\n", cheapest));
        }
        if let Some(fastest) = &results.summary.fastest_model {
            report.push_str(&format!("- **Fastest Model**: {}\n", fastest));
        }
        
        report.push_str("\n## Detailed Results\n\n");
        
        for execution in &results.executions {
            report.push_str(&format!("### {} - {}\n\n", execution.provider, execution.model));
            report.push_str(&format!("**Status**: {:?}\n", execution.status));
            report.push_str(&format!("**Response Time**: {}ms\n", execution.metadata.response_time_ms));
            report.push_str(&format!("**Cost**: ${:.6}\n", execution.metadata.cost_usd));
            report.push_str(&format!("**Tokens**: {} in, {} out\n\n", 
                execution.metadata.token_count_input, 
                execution.metadata.token_count_output));
            
            if matches!(execution.status, crate::core::evaluation::ExecutionStatus::Success) {
                report.push_str("**Output**:\n```\n");
                report.push_str(&execution.output);
                report.push_str("\n```\n\n");
            } else if let Some(error) = &execution.metadata.error {
                report.push_str(&format!("**Error**: {}\n\n", error));
            }
            
            report.push_str("---\n\n");
        }
    }
    
    report
}

async fn generate_html_report(evaluation: &Evaluation) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>EvalEds Report - {}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }}
        .header {{ border-bottom: 2px solid #e5e7eb; padding-bottom: 20px; margin-bottom: 30px; }}
        .summary {{ background: #f9fafb; padding: 20px; border-radius: 8px; margin-bottom: 30px; }}
        .execution {{ border: 1px solid #e5e7eb; padding: 20px; margin-bottom: 20px; border-radius: 8px; }}
        .success {{ border-left: 4px solid #10b981; }}
        .failed {{ border-left: 4px solid #ef4444; }}
        pre {{ background: #f3f4f6; padding: 15px; border-radius: 4px; overflow-x: auto; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; }}
        .metric {{ text-align: center; }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #1f2937; }}
        .metric-label {{ color: #6b7280; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Evaluation Report: {}</h1>
        <p>Generated on {}</p>
    </div>
</body>
</html>"#,
        evaluation.name,
        evaluation.name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    )
}

// Template constants
const LAYOUT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}}</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
            background: #f8fafc; color: #1e293b; line-height: 1.6;
        }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { 
            background: white; padding: 20px; border-radius: 8px; 
            box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 20px;
        }
        .nav { display: flex; gap: 20px; margin-top: 15px; }
        .nav a { 
            color: #64748b; text-decoration: none; padding: 8px 16px; 
            border-radius: 4px; transition: all 0.2s;
        }
        .nav a:hover { background: #f1f5f9; color: #0f172a; }
        .nav a.active { background: #3b82f6; color: white; }
        .card { 
            background: white; padding: 24px; border-radius: 8px; 
            box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 20px;
        }
        .metric { text-align: center; }
        .metric-value { font-size: 32px; font-weight: bold; color: #059669; }
        .metric-label { color: #64748b; margin-top: 4px; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }
        .execution-card { 
            border: 1px solid #e2e8f0; border-radius: 6px; 
            overflow: hidden; margin-bottom: 16px;
        }
        .execution-header { 
            background: #f8fafc; padding: 12px 16px; 
            border-bottom: 1px solid #e2e8f0;
        }
        .execution-content { padding: 16px; }
        .status-success { color: #059669; }
        .status-failed { color: #dc2626; }
        .btn { 
            background: #3b82f6; color: white; padding: 8px 16px; 
            border: none; border-radius: 4px; cursor: pointer; text-decoration: none;
            display: inline-block; margin-right: 8px;
        }
        .btn:hover { background: #2563eb; }
        .btn-secondary { background: #64748b; }
        .btn-secondary:hover { background: #475569; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{{evaluation.name}}</h1>
            {{#if evaluation.description}}
            <p>{{evaluation.description}}</p>
            {{/if}}
            <nav class="nav">
                <a href="/" {{#eq page "dashboard"}}class="active"{{/eq}}>📊 Dashboard</a>
                <a href="/comparison" {{#eq page "comparison"}}class="active"{{/eq}}>🔄 Comparison</a>
                <a href="/analysis" {{#eq page "analysis"}}class="active"{{/eq}}>🔍 Analysis</a>
            </nav>
        </div>
        {{{body}}}
    </div>
</body>
</html>"#;

const DASHBOARD_TEMPLATE: &str = r#"{{#*inline "page"}}
<div class="card">
    <h2>Evaluation Summary</h2>
    {{#if evaluation.results}}
    <div class="grid">
        <div class="metric">
            <div class="metric-value">{{evaluation.results.summary.total_executions}}</div>
            <div class="metric-label">Total Executions</div>
        </div>
        <div class="metric">
            <div class="metric-value">{{evaluation.results.summary.success_rate}}%</div>
            <div class="metric-label">Success Rate</div>
        </div>
        <div class="metric">
            <div class="metric-value">${{evaluation.results.summary.total_cost}}</div>
            <div class="metric-label">Total Cost</div>
        </div>
        <div class="metric">
            <div class="metric-value">{{evaluation.results.summary.avg_response_time}}ms</div>
            <div class="metric-label">Avg Response Time</div>
        </div>
    </div>
    {{/if}}
</div>

<div class="card">
    <h2>Quick Actions</h2>
    <a href="/comparison" class="btn">🔄 Compare Results</a>
    <a href="/analysis" class="btn">🔍 View Analysis</a>
    <a href="/export?format=markdown" class="btn btn-secondary">📄 Export Markdown</a>
    <a href="/export?format=json" class="btn btn-secondary">💾 Export JSON</a>
</div>

{{#if evaluation.results}}
<div class="card">
    <h2>Recent Executions</h2>
    {{#each evaluation.results.executions}}
    <div class="execution-card">
        <div class="execution-header">
            <strong>{{provider}}/{{model}}</strong>
            <span class="status-{{status}}">{{status}}</span>
        </div>
        <div class="execution-content">
            <p><strong>Response Time:</strong> {{metadata.response_time_ms}}ms</p>
            <p><strong>Cost:</strong> ${{metadata.cost_usd}}</p>
            <p><strong>Tokens:</strong> {{metadata.token_count_input}} in, {{metadata.token_count_output}} out</p>
        </div>
    </div>
    {{/each}}
</div>
{{/if}}
{{/inline}}

{{> layout}}"#;

const COMPARISON_TEMPLATE: &str = r#"{{#*inline "page"}}
<div class="card">
    <h2>Model Comparison</h2>
    {{#if evaluation.results}}
    <div class="grid">
        {{#each evaluation.results.executions}}
        <div class="execution-card">
            <div class="execution-header">
                <h3>{{provider}}/{{model}}</h3>
                <span class="status-{{status}}">{{status}}</span>
            </div>
            <div class="execution-content">
                <div style="margin-bottom: 12px;">
                    <span><strong>⏱️</strong> {{metadata.response_time_ms}}ms</span>
                    <span style="margin-left: 16px;"><strong>💰</strong> ${{metadata.cost_usd}}</span>
                </div>
                <div style="background: #f8fafc; padding: 12px; border-radius: 4px; font-size: 14px;">
                    {{#if output}}
                    {{output}}
                    {{else}}
                    <em>{{metadata.error}}</em>
                    {{/if}}
                </div>
            </div>
        </div>
        {{/each}}
    </div>
    {{else}}
    <p>No results available yet. Run the evaluation first.</p>
    {{/if}}
</div>
{{/inline}}

{{> layout}}"#;

const ANALYSIS_TEMPLATE: &str = r#"{{#*inline "page"}}
<div class="card">
    <h2>Performance Analysis</h2>
    {{#if evaluation.results}}
    <div class="grid">
        {{#if evaluation.results.summary.best_performing_model}}
        <div class="metric">
            <div class="metric-value">🏆</div>
            <div class="metric-label">Best Performing<br>{{evaluation.results.summary.best_performing_model}}</div>
        </div>
        {{/if}}
        {{#if evaluation.results.summary.most_cost_effective}}
        <div class="metric">
            <div class="metric-value">💰</div>
            <div class="metric-label">Most Cost Effective<br>{{evaluation.results.summary.most_cost_effective}}</div>
        </div>
        {{/if}}
        {{#if evaluation.results.summary.fastest_model}}
        <div class="metric">
            <div class="metric-value">⚡</div>
            <div class="metric-label">Fastest Model<br>{{evaluation.results.summary.fastest_model}}</div>
        </div>
        {{/if}}
    </div>
    {{/if}}
</div>

<div class="card">
    <h2>Detailed Metrics</h2>
    {{#if evaluation.results}}
    <table style="width: 100%; border-collapse: collapse;">
        <thead>
            <tr style="border-bottom: 2px solid #e2e8f0;">
                <th style="text-align: left; padding: 12px;">Provider/Model</th>
                <th style="text-align: right; padding: 12px;">Response Time</th>
                <th style="text-align: right; padding: 12px;">Cost</th>
                <th style="text-align: right; padding: 12px;">Tokens</th>
                <th style="text-align: center; padding: 12px;">Status</th>
            </tr>
        </thead>
        <tbody>
            {{#each evaluation.results.executions}}
            <tr style="border-bottom: 1px solid #f1f5f9;">
                <td style="padding: 12px; font-weight: 500;">{{provider}}/{{model}}</td>
                <td style="padding: 12px; text-align: right;">{{metadata.response_time_ms}}ms</td>
                <td style="padding: 12px; text-align: right;">${{metadata.cost_usd}}</td>
                <td style="padding: 12px; text-align: right;">{{metadata.token_count_output}}</td>
                <td style="padding: 12px; text-align: center;">
                    <span class="status-{{status}}">{{status}}</span>
                </td>
            </tr>
            {{/each}}
        </tbody>
    </table>
    {{else}}
    <p>No analysis data available yet.</p>
    {{/if}}
</div>
{{/inline}}

{{> layout}}"#;