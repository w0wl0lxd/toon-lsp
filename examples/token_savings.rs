//! Prints a byte-size and LLM-token comparison of the same logical document
//! serialized to TOON, JSON, YAML, and TOML.
//!
//! Run with: `cargo run --example token_savings --release`
//!
//! Token counts use the `o200k_base` encoding (tiktoken-rs), which is the
//! tokenizer shared by the current OpenAI models: GPT-5.x, GPT-4.1, GPT-4.5,
//! and the o-series / codex families.

use serde_json::Value;
use tiktoken_rs::o200k_base;
use toon_lsp::toon::encode;

fn sample() -> Value {
    serde_json::json!({
        "service": {
            "name": "gateway",
            "host": "0.0.0.0",
            "port": 8080,
            "replicas": 3,
            "enabled": true
        },
        "database": {
            "url": "postgres://localhost:5432/app",
            "pool_size": 16,
            "ssl": true,
            "migrations": true
        },
        "logging": {
            "level": "info",
            "format": "json",
            "outputs": ["stdout", "file"],
            "file": "/var/log/app.log"
        },
        "features": ["auth", "rate-limit", "metrics", "tracing"],
        "users": [
            {"id": 1, "name": "alice", "roles": ["admin", "dev"]},
            {"id": 2, "name": "bob", "roles": ["viewer"]},
            {"id": 3, "name": "carol", "roles": ["admin", "qa"]}
        ],
        "timeouts": {"connect_ms": 5000, "read_ms": 30000, "idle_ms": 60000},
        "metadata": {"version": "1.4.2", "env": "production", "region": "us-east-1"}
    })
}

fn long_sample() -> Value {
    serde_json::json!({
        "system_prompt": "You are a helpful assistant that answers questions about the \
deployment configuration. Always respond with the exact key names shown in the config and \
never invent new fields. When a value is a reference, resolve it before answering.",
        "description": "Primary ingress configuration for the production gateway. This block \
controls TLS termination, upstream routing, and rate limiting for all external traffic.",
        "notes": [
            "Enable HTTP/2 on the edge.",
            "Rotate certificates every 90 days via the cert-manager integration.",
            "Alert on p99 latency above 250ms for a rolling 5 minute window."
        ],
        "endpoints": [
            {"path": "/healthz", "method": "GET", "auth": false},
            {"path": "/api/v1/users", "method": "POST", "auth": true},
            {"path": "/api/v1/orders", "method": "PUT", "auth": true}
        ],
        "tags": {"team": "platform", "cost_center": "cc-4412", "tier": "gold"}
    })
}

fn report(name: &str, v: &Value) {
    let toon = encode(v).unwrap();
    let json = serde_json::to_string(v).unwrap();
    let yaml = serde_yaml::to_string(v).unwrap();
    let toml_s = toml::to_string(v).unwrap();

    let bpe = o200k_base().expect("failed to load o200k_base tokenizer");

    let rows = [
        ("TOON", toon.clone()),
        ("JSON", json.clone()),
        ("YAML", yaml.clone()),
        ("TOML", toml_s.clone()),
    ];

    println!("\n=== {name} ===\n");
    println!("| Format | Bytes | Tokens |");
    println!("|---|---:|---:|");

    let json_tokens = bpe.count_ordinary(&json);
    for (n, s) in &rows {
        let tokens = bpe.count_ordinary(s);
        let vs = if *n == "TOON" {
            String::new()
        } else {
            let pct = (1.0 - tokens as f64 / json_tokens as f64) * 100.0;
            format!(" ({pct:+.0}%)")
        };
        println!("| {n} | {} | {}{} |", s.len(), tokens, vs);
    }
    let toon_tokens = bpe.count_ordinary(&toon);
    println!(
        "\nTOON vs JSON: {:.0}% bytes, {:.0}% tokens",
        (toon.len() as f64 / json.len() as f64) * 100.0,
        (toon_tokens as f64 / json_tokens as f64) * 100.0
    );
}

fn main() {
    report("compact config", &sample());
    report("long-text prompts", &long_sample());
}
