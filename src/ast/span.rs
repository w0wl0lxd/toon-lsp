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
    /// Create a new position.
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self { line, column, offset }
    }

    /// Create position at start of file.
    pub fn start() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
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
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a zero-width span at a position.
    pub fn point(pos: Position) -> Self {
        Self::new(pos, pos)
    }

    /// Check if this span contains a position.
    pub fn contains(&self, pos: Position) -> bool {
        pos.offset >= self.start.offset && pos.offset < self.end.offset
    }

    /// Merge two spans into one that covers both.
    pub fn merge(self, other: Span) -> Span {
        let start = if self.start.offset <= other.start.offset {
            self.start
        } else {
            other.start
        };
        let end = if self.end.offset >= other.end.offset {
            self.end
        } else {
            other.end
        };
        Span::new(start, end)
    }

    /// Get the length of this span in bytes.
    pub fn len(&self) -> u32 {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if this span is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::point(Position::default())
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
        let span = Span::new(
            Position::new(0, 0, 0),
            Position::new(0, 10, 10),
        );
        assert!(span.contains(Position::new(0, 5, 5)));
        assert!(!span.contains(Position::new(0, 15, 15)));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(
            Position::new(0, 0, 0),
            Position::new(0, 5, 5),
        );
        let span2 = Span::new(
            Position::new(0, 3, 3),
            Position::new(0, 10, 10),
        );
        let merged = span1.merge(span2);
        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 10);
    }
}
