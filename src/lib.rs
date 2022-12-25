// SPDX-License-Identifier: GPL-2.0
//
// This file is part of chscn.
// chscn is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 2
// of the License, or (at your option) any later version.
//
// chscn is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Foobar.
// If not, see <https://www.gnu.org/licenses/>.

use std::str::Chars;

/// Base type for line and column numbers.
pub type Counter = u32;

/// Position in a text by line and column numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    /// line number (starts counting with 1)
    pub line: Counter,
    /// character number within the current line (starts counting with 1)
    pub column: Counter,
}

impl Position {
    /// Creates an invalid `Position` (e.g. line = 0, column = 0).
    pub fn new() -> Self {
        Position::with( 0, 0)
    }

    /// Creates a new `Position` with the given line and column number.
    pub fn with( line: Counter, column: Counter ) -> Self {
        Position{ line, column }
    }

    /// Advance position by one (non-new-line) character.
    pub fn advance_char(&mut self) {
        self.column += 1
    }

    /// Advance position by one line. Sets the column to the position of the first character
    /// within the new line.
    pub fn advance_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }
}

#[derive(Clone, Debug)]
pub struct Text<'a> {
    iter: Chars<'a>,
    position: Position, // position of NEXT character to be returned by `next()`
    next: Option<char>,
    marker: Option<Chars<'a>>,
    last_was_cr: bool,
}

impl<'a> Text<'a> {
    /// Creates a new `Text` that wraps the given source text.
    pub fn with_str(text: &'a str) -> Self {
        Text { iter: text.chars(), position: Position::with(1,1),
            next: None, marker: None, last_was_cr: false }
    }

    /// Returns the position of the NEXT character that will be returned by `next()`
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Returns the next char or None if EOF, but does not consume the character.
    /// The position will not be updated.
    pub fn peek_next(&mut self) -> Option<char> {
        if self.next.is_none() {
            self.next = self.iter.next();
        }
        self.next.clone()
    }

    /// Sets the marker at the current reading position.
    pub fn set_marker(&mut self) {
        self.marker = Some( self.iter.clone() );
    }

    /// Clears the actual marker if one is stored.
    pub fn clear_marker(&mut self) {
        self.marker = None;
    }

    /// Returns `true` when a marker is actually set.
    pub fn has_marker(&self) -> bool {
        self.marker.is_some()
    }

    /// Returns a string slice reference from the set marker up to (excluding) the current reading
    /// position.
    pub fn slice_from_marker(&self) -> &'a str {
        assert!(self.has_marker());
        let s = self.marker.as_ref().unwrap().as_str();
        let len = if let Some(ch) = self.next.as_ref() {
            s.len() - self.iter.as_str().len() - ch.len_utf8()
        }
        else {
            s.len() - self.iter.as_str().len()
        };

        assert!( len <= s.len() );
        s.get(0.. len).unwrap()
    }

    fn advance_position(&mut self, ch: &char) {
        match ch {
            '\r' => {
                self.last_was_cr = true;
                self.position.advance_line();
            },
            '\n' => {
                if !self.last_was_cr {
                    self.position.advance_line();
                    self.last_was_cr = false;
                }
            },
            '\u{000b}' => {
                self.position.line += 1;
                self.last_was_cr = false;
            },
            '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}' => {
                self.last_was_cr = false;
                self.position.advance_line();
            },
            _ => {
                self.last_was_cr = false;
                self.position.advance_char();
            }
        }
    }
}


impl<'a> Iterator for Text<'a> {
    type Item = char;

    /// Returns the next character or None if the file is at its end.
    /// The position will be updated according to the read character.
    fn next(&mut self) -> Option<Self::Item> {
        let ch = if self.next.is_some() {
            self.next.take()
        }
        else {
            self.iter.next()
        };

        if let Some(nch)  = ch.as_ref() {
            self.advance_position(nch);
        }
        ch
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_marker() {
        let mut text = Text::with_str( " some_value_ 1" );

        let _ = text.next();
        text.set_marker();
        loop {
            match text.peek_next() {
                Some(' ') => break,
                Some( _ ) => { let _ = text.next(); },
                None => break,
            }
        }
        let s = text.slice_from_marker();
        assert_eq!(s, "some_value_");
    }


    #[test]
    fn text_peek() {
        let src = "This is my text\nwith three lines.\n";
        let mut text = Text::with_str(src);

        let _ = text.next();
        let _ = text.next();
        let _ = text.next();

        assert_eq!(text.position(), &Position::with(1, 4));
        assert_eq!(text.peek_next(), Some('s'));
        assert_eq!(text.position(), &Position::with(1, 4));
        assert_eq!(text.peek_next(), Some('s'));
        assert_eq!(text.position(), &Position::with(1, 4));
        assert_eq!(text.next(), Some('s'));
        assert_eq!(text.position(), &Position::with(1, 5));

        assert_eq!(text.peek_next(), Some(' '));
        assert_eq!(text.next(), Some(' '));
    }

    #[test]
    fn text_iterate() {
        let src = "This is my text\nwith three lines.\n";
        let mut text = Text::with_str(src);

        assert_eq!(text.position(), &Position::with(1,1));
        assert_eq!(text.next(), Some( 'T' ));

        assert_eq!(text.position(), &Position::with(1,2));
        assert_eq!(text.next(), Some( 'h' ));
        assert_eq!(text.next(), Some( 'i' ));
        assert_eq!(text.next(), Some( 's' ));
        assert_eq!(text.next(), Some( ' ' ));
        assert_eq!(text.next(), Some( 'i' ));
        assert_eq!(text.next(), Some( 's' ));
        assert_eq!(text.next(), Some( ' ' ));
        assert_eq!(text.next(), Some( 'm' ));
        assert_eq!(text.next(), Some( 'y' ));
        assert_eq!(text.next(), Some( ' ' ));
        assert_eq!(text.next(), Some( 't' ));
        assert_eq!(text.next(), Some( 'e' ));
        assert_eq!(text.next(), Some( 'x' ));
        assert_eq!(text.next(), Some( 't' ));

        assert_eq!(text.position(), &Position::with(1, 16));
        assert_eq!(text.next(), Some( '\n' ));

        assert_eq!(text.position(), &Position::with(2, 1))
    }
}
