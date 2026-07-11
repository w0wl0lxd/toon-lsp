// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Integration tests for the graph command.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Get a command for the toon-lsp binary
#[allow(deprecated)]
fn toon_lsp() -> Command {
    Command::cargo_bin("toon-lsp").expect("should build binary")
}

#[test]
fn test_graph_simple_dependency() {
    // Given: TOON file with a reference dependency
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("dependency.toon");
    fs::write(&toon_path, "db:\n  port: 5432\nconnection:\n  url: ${db.port}\n")
        .expect("write file");

    // When: User runs graph
    let mut cmd = toon_lsp();
    cmd.arg("graph").arg(&toon_path);

    // Then: Dependency edge is in the Mermaid graph output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("flowchart TD"))
        .stdout(predicate::str::contains("db.port"))
        .stdout(predicate::str::contains("connection.url"))
        .stdout(predicate::str::contains("-->"));
}
