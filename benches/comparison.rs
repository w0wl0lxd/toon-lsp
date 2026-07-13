use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use serde_json::Value;
use toon_lsp::toon::{decode, encode};

/// A realistic service-config document used as the shared benchmark input.
/// Every format parser is benchmarked against this same logical data.
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

fn bench_parse_throughput(c: &mut Criterion) {
    let v = sample();
    let toon = encode(&v).unwrap();
    let json = serde_json::to_string(&v).unwrap();
    let yaml = serde_yaml::to_string(&v).unwrap();
    let toml_s = toml::to_string(&v).unwrap();
    // JSON is a strict subset of JSON5, so the same compact string is valid JSON5 input.
    let json5 = json.clone();

    let cases: &[(&str, &str, &dyn Fn(&str))] = &[
        ("toon/decode", &toon, &|s| {
            let _ = decode(s).unwrap();
        }),
        ("json", &json, &|s| {
            let _: Value = serde_json::from_str(s).unwrap();
        }),
        ("yaml", &yaml, &|s| {
            let _: Value = serde_yaml::from_str(s).unwrap();
        }),
        ("toml", &toml_s, &|s| {
            let _: Value = toml::from_str(s).unwrap();
        }),
        ("json5", &json5, &|s| {
            let _: Value = json5::from_str(s).unwrap();
        }),
    ];

    for (name, input, run) in cases {
        let mut g = c.benchmark_group(format!("parse/{name}"));
        g.throughput(Throughput::Bytes(input.len() as u64));
        g.bench_with_input(BenchmarkId::new("throughput", ""), input, |b, inp| {
            b.iter(|| run(inp));
        });
        g.finish();
    }
}

criterion_group!(benches, bench_parse_throughput);
criterion_main!(benches);
