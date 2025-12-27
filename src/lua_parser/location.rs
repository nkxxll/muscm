//! Location tracking for tokens and AST nodes

use super::Token;

/// Source location information (line and column numbers)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    /// 1-based line number
    pub line: usize,
    /// 0-based column number (position in the line)
    pub column: usize,
}

impl Location {
    /// Create a new location
    pub fn new(line: usize, column: usize) -> Self {
        Location { line, column }
    }

    /// Create a location at the start of a file
    pub fn start() -> Self {
        Location { line: 1, column: 0 }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A token paired with its source location
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenWithLocation {
    pub token: Token,
    pub location: Location,
}

impl TokenWithLocation {
    /// Create a new token with location
    pub fn new(token: Token, location: Location) -> Self {
        TokenWithLocation { token, location }
    }
}

/// Helper to track location while processing source code
pub struct LocationTracker {
    line: usize,
    column: usize,
}

impl LocationTracker {
    /// Create a new location tracker starting at line 1, column 0
    pub fn new() -> Self {
        LocationTracker { line: 1, column: 0 }
    }

    /// Get current location
    pub fn current(&self) -> Location {
        Location {
            line: self.line,
            column: self.column,
        }
    }

    /// Record processing a character
    pub fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
    }

    /// Record processing multiple characters
    pub fn advance_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.advance(ch);
        }
    }

    /// Skip whitespace and comments, returning the number of characters consumed
    pub fn skip_whitespace_and_comments(&mut self, input: &str) -> usize {
        let mut consumed = 0;
        let mut remaining = input;

        loop {
            // Skip comments
            if remaining.starts_with("--") {
                if let Some(newline_pos) = remaining.find('\n') {
                    self.advance_str(&remaining[..newline_pos + 1]);
                    consumed += newline_pos + 1;
                    remaining = &remaining[newline_pos + 1..];
                } else {
                    self.advance_str(remaining);
                    consumed += remaining.len();
                    remaining = "";
                }
            } else if remaining.chars().next().is_some_and(char::is_whitespace) {
                let ch = remaining.chars().next().unwrap();
                self.advance(ch);
                consumed += ch.len_utf8();
                remaining = &remaining[ch.len_utf8()..];
            } else {
                break;
            }
        }

        consumed
    }
}

impl Default for LocationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_creation() {
        let loc = Location::new(5, 10);
        assert_eq!(loc.line, 5);
        assert_eq!(loc.column, 10);
    }

    #[test]
    fn test_location_start() {
        let loc = Location::start();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 0);
    }

    #[test]
    fn test_location_display() {
        let loc = Location::new(42, 15);
        assert_eq!(loc.to_string(), "42:15");
    }

    #[test]
    fn test_location_tracker() {
        let mut tracker = LocationTracker::new();
        assert_eq!(tracker.current(), Location::new(1, 0));

        tracker.advance('a');
        assert_eq!(tracker.current(), Location::new(1, 1));

        tracker.advance('\n');
        assert_eq!(tracker.current(), Location::new(2, 0));

        tracker.advance('b');
        assert_eq!(tracker.current(), Location::new(2, 1));
    }

    #[test]
    fn test_location_tracker_advance_str() {
        let mut tracker = LocationTracker::new();
        tracker.advance_str("hello");
        assert_eq!(tracker.current(), Location::new(1, 5));

        tracker.advance_str("\nworld");
        assert_eq!(tracker.current(), Location::new(2, 5));
    }

    #[test]
    fn test_location_tracker_skip_whitespace() {
        let mut tracker = LocationTracker::new();
        let input = "  \n  hello";
        let consumed = tracker.skip_whitespace_and_comments(input);
        assert_eq!(consumed, 5);
        assert_eq!(tracker.current(), Location::new(2, 2));
    }

    #[test]
    fn test_location_tracker_skip_comments() {
        let mut tracker = LocationTracker::new();
        let input = "-- comment\nhello";
        let consumed = tracker.skip_whitespace_and_comments(input);
        assert_eq!(consumed, 11);
        assert_eq!(tracker.current(), Location::new(2, 0));
    }

    #[test]
    fn test_token_with_location() {
        let tok = TokenWithLocation::new(Token::True, Location::new(5, 10));
        assert_eq!(tok.location.line, 5);
        assert_eq!(tok.location.column, 10);
    }
}
