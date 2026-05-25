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

//! Source position and span types for tracking locations in TOON documents.

use serde::{Deserialize, Serialize};

/// A position in a source file (0-indexed line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: u32,
    /// Column number (0-indexed, in UTF-8 code units)
    pub column: u32,
    /// Byte offset from start of file
    pub offset: u32,
}

impl Position {
    /// Position at start of file (0, 0, 0).
    pub const ZERO: Self = Self { line: 0, column: 0, offset: 0 };

    /// Create a new position.
    #[inline]
    pub const fn new(line: u32, column: u32, offset: u32) -> Self {
        Self { line, column, offset }
    }
}

impl Default for Position {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

/// A span representing a range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Span {
    /// Create a new span from start to end positions.
    #[inline]
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a zero-width span at a position.
    #[inline]
    pub const fn point(pos: Position) -> Self {
        Self { start: pos, end: pos }
    }

    /// Check if this span contains a position.
    #[inline]
    #[must_use]
    pub fn contains(&self, pos: Position) -> bool {
        (self.start.offset..self.end.offset).contains(&pos.offset)
    }

    /// Merge two spans into one that covers both.
    #[inline]
    #[must_use]
    pub fn merge(self, other: Span) -> Span {
        Span::new(
            Position::new(
                self.start.line.min(other.start.line),
                self.start.column.min(other.start.column),
                self.start.offset.min(other.start.offset),
            ),
            Position::new(
                self.end.line.max(other.end.line),
                self.end.column.max(other.end.column),
                self.end.offset.max(other.end.offset),
            ),
        )
    }

    /// Get the length of this span in bytes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> u32 {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if this span is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start.offset >= self.end.offset
    }
}

impl Default for Span {
    #[inline]
    fn default() -> Self {
        Self::point(Position::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(1, 5, 10);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 10);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(Position::new(0, 0, 0), Position::new(0, 10, 10));
        assert!(span.contains(Position::new(0, 5, 5)));
        assert!(!span.contains(Position::new(0, 15, 15)));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(Position::new(0, 0, 0), Position::new(0, 5, 5));
        let span2 = Span::new(Position::new(0, 3, 3), Position::new(0, 10, 10));
        let merged = span1.merge(span2);
        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 10);
    }
}
