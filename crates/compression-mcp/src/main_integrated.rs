/// Example: Using Integrated Compression Manager
///
/// This shows how to use all 4 phases in a unified, simple API.
/// This replaces main.rs when using the fully integrated system.

use compression_mcp::{IntegratedCompressionManager, IntegratedConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Initialize integrated manager with all phases enabled
    let config = IntegratedConfig {
        auto_compress_enabled: true,
        compress_threshold: 1000,
        excluded_tools: vec![],
        enable_personalization: true,
        learn_from_history: true,
        enable_persistent_storage: true,
        storage_path: "./headroom.db".to_string(),
        ccr_retention_days: 30,
        safety_level: "moderate".to_string(),
        verbose_logging: false,
    };

    let manager = IntegratedCompressionManager::new(config)?;

    // Health check: all phases operational
    let health = manager.health_check()?;
    println!("Health Status: {:?}", health);

    // Example 1: Simple compression (all phases work together)
    println!("\n=== Example 1: Simple Compression ===");
    let agent_id = "agent-42";
    let tool_name = "shell";
    let raw_output = r#"
        {
            "status": "ok",
            "error": null,
            "metadata": {},
            "result": "success",
            "timestamp": "2026-07-05T10:30:45Z",
            "retry_count": 3
        }
    "#;

    let result = manager.compress(agent_id, tool_name, raw_output)?;
    println!("Compressed: {} → {} ({:.2}x, {} tokens saved)",
        result.original_output.len(),
        result.compressed_output.len(),
        result.compression_ratio,
        result.tokens_saved
    );

    // Example 2: Retrieve original (Phase 1/4)
    println!("\n=== Example 2: Retrieve Original ===");
    let original = manager.retrieve(&result.output_id)?;
    println!("Retrieved {} bytes (byte-equal: {})",
        original.len(),
        original == result.original_output
    );

    // Example 3: Record task result for personalization (Phase 3)
    println!("\n=== Example 3: Personalization Learning ===");
    manager.record_task_result(agent_id, true, 0.95, result.tokens_saved)?;
    let strategy = manager.get_agent_strategy(agent_id)?;
    println!("Agent strategy: {}", strategy);

    // Example 4: Get metrics across all phases (Phase 2, 3, 4)
    println!("\n=== Example 4: Metrics & Analytics ===");
    let metrics = manager.get_metrics_snapshot();
    println!("{}", metrics);

    // Example 5: Export metrics in multiple formats
    println!("\n=== Example 5: Export Metrics ===");
    let prometheus = manager.export_metrics("prometheus")?;
    println!("Prometheus export:\n{}", prometheus);

    // Example 6: Identify top agents (Phase 3)
    println!("\n=== Example 6: Top Agents ===");
    let top = manager.get_top_agents(5)?;
    println!("Top agents: {:?}", top);

    // Example 7: Handler loop (Phase 2 - automatic hooks)
    println!("\n=== Example 7: Hook Handler Loop ===");
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut writer = tokio::io::BufWriter::new(stdout);
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    // Send server capabilities
    let server_info = json!({
        "type": "integrated_compression_server",
        "version": "1.0",
        "phases": {
            "phase_1": "Foundation - Manual compression with CCR",
            "phase_2": "Automatic hooks - Transparent compression",
            "phase_3": "Personalization - Per-agent adaptive strategies",
            "phase_4": "Persistence - Multi-session learning"
        },
        "capabilities": {
            "compress": "Compress tool output with all phases",
            "retrieve": "Retrieve original via CCR",
            "metrics": "Get compression metrics",
            "strategies": "Get personalized strategies",
            "health": "Health check all phases"
        }
    });

    writer.write_all(server_info.to_string().as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // Read requests and handle (would implement full MCP protocol here)
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
            let response = match request.get("method").and_then(|v| v.as_str()) {
                Some("compress") => {
                    let params = request.get("params").cloned().unwrap_or_default();
                    match (
                        params.get("agent_id").and_then(|v| v.as_str()),
                        params.get("tool_name").and_then(|v| v.as_str()),
                        params.get("raw_output").and_then(|v| v.as_str()),
                    ) {
                        (Some(a), Some(t), Some(o)) => {
                            match manager.compress(a, t, o) {
                                Ok(result) => json!({
                                    "result": {
                                        "output_id": result.output_id,
                                        "compressed_output": result.compressed_output,
                                        "compression_ratio": result.compression_ratio,
                                        "tokens_saved": result.tokens_saved,
                                        "content_type": format!("{:?}", result.content_type),
                                    }
                                }),
                                Err(e) => json!({"error": e}),
                            }
                        }
                        _ => json!({"error": "Missing parameters"}),
                    }
                }
                Some("health") => {
                    match manager.health_check() {
                        Ok(status) => json!({
                            "result": {
                                "phase1": status.phase1_operational,
                                "phase2": status.phase2_operational,
                                "phase3": status.phase3_operational,
                                "phase4": status.phase4_operational,
                                "all_operational": status.all_operational,
                            }
                        }),
                        Err(e) => json!({"error": e}),
                    }
                }
                _ => json!({"error": "Unknown method"}),
            };

            writer.write_all(response.to_string().as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
        line.clear();
    }

    Ok(())
}
