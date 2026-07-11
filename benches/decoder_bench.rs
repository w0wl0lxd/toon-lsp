use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use toon_lsp::toon::{decode, encode};

fn bench_decode(c: &mut Criterion) {
    let toon = encode(&json!({
        "users": [
            {"id": 1, "name": "Alice", "tags": ["admin", "user"]},
            {"id": 2, "name": "Bob", "tags": ["user"]},
            {"id": 3, "name": "Carol", "tags": ["admin", "user", "guest"]}
        ],
        "config": {
            "server": {"host": "localhost", "port": 8080},
            "database": {"url": "postgres://localhost/db", "pool": 10}
        },
        "metadata": {"version": "1.0", "features": ["auth", "logging"]}
    })).unwrap();

    let mut group = c.benchmark_group("decode");
    group.bench_function("decode_complex", |b| {
        b.iter(|| decode(black_box(&toon)).unwrap())
    });
    group.finish();
}

fn bench_encode(c: &mut Criterion) {
    let value = json!({
        "users": [
            {"id": 1, "name": "Alice", "tags": ["admin", "user"]},
            {"id": 2, "name": "Bob", "tags": ["user"]},
            {"id": 3, "name": "Carol", "tags": ["admin", "user", "guest"]}
        ],
        "config": {
            "server": {"host": "localhost", "port": 8080},
            "database": {"url": "postgres://localhost/db", "pool": 10}
        },
        "metadata": {"version": "1.0", "features": ["auth", "logging"]}
    });

    let mut group = c.benchmark_group("encode");
    group.bench_function("encode_complex", |b| {
        b.iter(|| encode(black_box(&value)).unwrap())
    });
    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let value = json!({
        "users": [
            {"id": 1, "name": "Alice", "tags": ["admin", "user"]},
            {"id": 2, "name": "Bob", "tags": ["user"]},
            {"id": 3, "name": "Carol", "tags": ["admin", "user", "guest"]}
        ],
        "config": {
            "server": {"host": "localhost", "port": 8080},
            "database": {"url": "postgres://localhost/db", "pool": 10}
        },
        "metadata": {"version": "1.0", "features": ["auth", "logging"]}
    });

    let mut group = c.benchmark_group("roundtrip");
    group.bench_function("roundtrip_complex", |b| {
        b.iter(|| {
            let encoded = encode(black_box(&value)).unwrap();
            decode(black_box(&encoded)).unwrap()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_decode, bench_encode, bench_roundtrip);
criterion_main!(benches);