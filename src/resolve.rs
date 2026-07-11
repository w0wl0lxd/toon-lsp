// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Reference resolution for TOON (`${path}` and `${env:VAR}`).
//!
//! References are resolved lazily against the document AST (by dotted path)
//! or the process environment (`env:` prefix). Cycle detection prevents
//! infinite recursion when references form a loop.

use crate::ast::{AstNode, ObjectEntry, Span};

/// Error encountered while resolving a reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// A path segment could not be found in the document.
    NotFound(String),
    /// Resolution would recurse infinitely (the chain is included).
    Cycle(Vec<String>),
    /// An `env:` reference named an unset (or invalid) environment variable.
    EnvNotSet(String),
}

/// The resolved target of a reference.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedRef<'a> {
    /// A node elsewhere in the document, with the span of the key that owns it.
    Node {
        /// The resolved value node.
        node: &'a AstNode,
        /// Span of the key that owns the resolved value (for goto-definition).
        key_span: Option<Span>,
    },
    /// A value sourced from the process environment.
    Env(String),
}

/// Resolve a reference by its raw interior (e.g. `foo.bar` or `env:VAR`).
///
/// Returns the resolved value (or its environment string) along with, for
/// document references, the span of the key that owns the value.
///
/// # Examples
///
/// ```
/// use toon_lsp::{parse, resolve::resolve};
///
/// let ast = parse("foo:\n  bar: 42\nref: ${foo.bar}").unwrap();
/// let resolved = resolve(&ast, "foo.bar").unwrap();
/// assert!(matches!(resolved, toon_lsp::resolve::ResolvedRef::Node { .. }));
/// ```
#[must_use]
pub fn resolve<'a>(root: &'a AstNode, raw: &str) -> Result<ResolvedRef<'a>, ResolveError> {
    let mut seen = Vec::new();
    resolve_inner(root, raw, &mut seen)
}

fn resolve_inner<'a>(
    root: &'a AstNode,
    raw: &str,
    seen: &mut Vec<String>,
) -> Result<ResolvedRef<'a>, ResolveError> {
    if seen.contains(&raw.to_string()) {
        return Err(ResolveError::Cycle(seen.clone()));
    }
    seen.push(raw.to_string());

    let result = if let Some(env_var) = raw.strip_prefix("env:") {
        match std::env::var(env_var) {
            Ok(value) => Ok(ResolvedRef::Env(value)),
            Err(_) => Err(ResolveError::EnvNotSet(env_var.to_string())),
        }
    } else {
        let segments: Vec<&str> = raw.split('.').collect();
        resolve_segments(root, &segments, seen)
    };

    seen.pop();
    result
}

/// Resolve a dotted path against the document, returning the final value node.
fn resolve_segments<'a>(
    root: &'a AstNode,
    segments: &[&str],
    seen: &mut Vec<String>,
) -> Result<ResolvedRef<'a>, ResolveError> {
    if segments.is_empty() {
        return Err(ResolveError::NotFound(String::new()));
    }

    let mut current: &AstNode = root;
    let last = segments.len() - 1;

    for (i, &seg) in segments.iter().enumerate() {
        let entries = entries_of(current)?;
        let entry = entries
            .iter()
            .find(|e| e.key == seg)
            .ok_or_else(|| ResolveError::NotFound(segments[..=i].join(".")))?;

        // Resolve the value if it is itself a reference (supports chains).
        let resolved = match &entry.value {
            AstNode::Reference { path, .. } => resolve_inner(root, path, seen)?,
            other => ResolvedRef::Node {
                node: other,
                key_span: Some(entry.key_span),
            },
        };

        match resolved {
            ResolvedRef::Node { node, key_span } => {
                current = node;
                if i == last {
                    return Ok(ResolvedRef::Node { node, key_span });
                }
            }
            ResolvedRef::Env(value) => return Ok(ResolvedRef::Env(value)),
        }
    }

    Ok(ResolvedRef::Node {
        node: current,
        key_span: None,
    })
}

/// Return the object entries reachable for path navigation from `node`.
///
/// A document's resolvable keys live in its single root object; an object
/// exposes its own entries directly.
fn entries_of<'a>(node: &'a AstNode) -> Result<&'a [ObjectEntry], ResolveError> {
    match node {
        AstNode::Document { children, .. } => children
            .iter()
            .find_map(|c| match c {
                AstNode::Object { entries, .. } => Some(entries.as_slice()),
                _ => None,
            })
            .ok_or_else(|| ResolveError::NotFound(String::new())),
        AstNode::Object { entries, .. } => Ok(entries),
        _ => Err(ResolveError::NotFound(String::new())),
    }
}

/// Collect all reference nodes in the document (for diagnostics).
#[must_use]
pub fn collect_references<'a>(node: &'a AstNode, out: &mut Vec<&'a AstNode>) {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                collect_references(child, out);
            }
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                collect_references(&entry.value, out);
            }
        }
        AstNode::Array { items, .. } => {
            for item in items {
                collect_references(item, out);
            }
        }
        AstNode::Reference { .. } => out.push(node),
        AstNode::String { .. }
        | AstNode::Number { .. }
        | AstNode::Bool { .. }
        | AstNode::Null { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn refs_of(src: &str) -> Vec<&'static str> {
        let ast = parse(src).expect("parse");
        let mut out = Vec::new();
        collect_references(&ast, &mut out);
        out.into_iter()
            .map(|n| match n {
                AstNode::Reference { path, .. } => path.as_str(),
                _ => unreachable!(),
            })
            .collect()
    }

    #[test]
    fn collects_top_level_reference() {
        assert_eq!(refs_of("ref: ${other}"), vec!["other"]);
    }

    #[test]
    fn collects_env_reference() {
        assert_eq!(refs_of("ref: ${env:HOME}"), vec!["env:HOME"]);
    }

    #[test]
    fn resolves_nested_path() {
        let ast = parse("foo:\n  bar: 42\nref: ${foo.bar}").unwrap();
        let resolved = resolve(&ast, "foo.bar").unwrap();
        match resolved {
            ResolvedRef::Node { node, .. } => {
                assert!(matches!(node, AstNode::Number { .. }));
            }
            _ => panic!("expected node"),
        }
    }

    #[test]
    fn resolves_env_when_set() {
        std::env::set_var("TOON_TEST_VAR", "hello");
        let ast = parse("ref: ${env:TOON_TEST_VAR}").unwrap();
        assert_eq!(resolve(&ast, "env:TOON_TEST_VAR").unwrap(), ResolvedRef::Env("hello".into()));
    }

    #[test]
    fn unresolved_path_is_not_found() {
        let ast = parse("ref: ${missing}").unwrap();
        assert_eq!(resolve(&ast, "missing"), Err(ResolveError::NotFound("missing".into())));
    }

    #[test]
    fn cycle_is_detected() {
        let ast = parse("a: ${b}\nb: ${a}").unwrap();
        assert!(matches!(resolve(&ast, "a"), Err(ResolveError::Cycle(_))));
    }
}
