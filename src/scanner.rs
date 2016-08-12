// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Scanner. Like a peekable iterator, but works on char values
//! instead of references.  Avoids some lifetime trouble.

/// A scanner holds a stream of characters and a current position.
pub struct Scanner {
    chars: Vec<char>,
    pos: usize,
    current: Option<char>,
}

impl Scanner {
    /// Create a new scanner from a string.  Initializes the current
    /// character to the first of the string, or None for an empty
    /// string.
    pub fn new(txt: &str) -> Scanner {
        let cs: Vec<_> = txt.chars().collect();
        let cur = cs.get(0).map(|cp| *cp);
        let s = Scanner {
            chars: cs,
            pos: 1,
            current: cur
        };
        s
    }

    /// Set the current character to the next one, or None when the end
    /// of the string is reached.
    pub fn next(&mut self) {
        if self.pos < self.chars.len() {
            self.current = self.chars.get(self.pos).map(|cp| *cp);
            self.pos += 1;
        } else {
            self.current = None;
        }
    }

    /// Delivers the current character or None at the end of the
    /// string.
    pub fn current(&self) -> Option<char> {
        self.current
    }
}

/// Skip the whitespace characters space, tab, lf and cr at the
/// current position.  After this function returns, the current
/// character is a non-whitespace character or the end of the string
/// is reached.
pub fn skip_ws(s: &mut Scanner) {
    loop {
        match s.current() {
            None =>
                return,
            Some(c) if c == ' ' || c == '\r' || c == '\n' || c == '\t' => {
                s.next();
            },
            Some(_) =>
                return,
        }
    }
}
