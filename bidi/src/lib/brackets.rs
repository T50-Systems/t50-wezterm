struct Pair {
    opening_pos: usize,
    closing_pos: usize,
}

impl core::fmt::Debug for Pair {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "Pair{{{},{}}}", self.opening_pos, self.closing_pos)
    }
}

const MAX_PAIRING_DEPTH: usize = 63;
struct BracketStack {
    closing_bracket: [char; MAX_PAIRING_DEPTH],
    position: [usize; MAX_PAIRING_DEPTH],
    depth: usize,
    pairs: Vec<Pair>,
}

impl core::fmt::Debug for BracketStack {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_struct("BracketStack")
            .field("closing_bracket", &&self.closing_bracket[0..self.depth])
            .field("position", &&self.position[0..self.depth])
            .field("depth", &self.depth)
            .field("pairs", &self.pairs)
            .finish()
    }
}

impl BracketStack {
    pub fn new() -> Self {
        Self {
            closing_bracket: [' '; MAX_PAIRING_DEPTH],
            position: [0; MAX_PAIRING_DEPTH],
            depth: 0,
            pairs: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
        self.depth = 0;
    }

    pub fn push(&mut self, closing_bracket: char, pos: usize) -> bool {
        let depth = self.depth;
        if depth >= MAX_PAIRING_DEPTH {
            return false;
        }
        self.closing_bracket[depth] = closing_bracket;
        self.position[depth] = pos;
        self.depth += 1;
        true
    }

    /// Seek an opening bracket pair for the closing bracket
    /// passed in.
    ///
    /// This is a stack based search.
    /// Start with the top element in the stack and search
    /// downwards until we either find a match or reach the
    /// bottom of the stack.
    ///
    /// If we find a match, construct and append the bracket
    /// pair to the pairList. Then pop the stack for all the
    /// levels down to the level where we found the match.
    /// (This approach is designed to discard pairs that
    /// are not cleanly nested.)
    ///
    /// If we search all the way to the bottom of the stack
    /// without finding a match, just return without changing
    /// state. This represents a closing bracket with no
    /// opening bracket to match it. Just discard and move on.
    pub fn seek_matching_open_bracket(&mut self, closing_bracket: char, pos: usize) -> bool {
        trace!(
            "seek_matching_open_bracket: closing_bracket={:?} pos={}\n{:?}",
            closing_bracket,
            pos,
            self
        );
        for depth in (0..self.depth).rev() {
            trace!("seek_matching_open_bracket: consider depth={}", depth);
            // The basic test is for the closingcp equal to the bpb value
            // stored in the bracketData. But to account for the canonical
            // equivalences for U+2329 and U+232A, tack on extra checks here
            // for the asymmetrical matches. This hard-coded check avoids
            // having to require full normalization of all the bracket code
            // points before checking. It is highly unlikely that additional
            // canonical singletons for bracket pairs will be added to future
            // versions of the UCD.
            if self.closing_bracket[depth] == closing_bracket
                || (self.closing_bracket[depth] == '\u{232a}' && closing_bracket == '\u{3009}')
                || (self.closing_bracket[depth] == '\u{3009}' && closing_bracket == '\u{232a}')
            {
                self.pairs.push(Pair {
                    opening_pos: self.position[depth],
                    closing_pos: pos,
                });
                // Pop back to this depth, pruning out any intermediates;
                // they are mismatched brackets
                self.depth = depth;
                return true;
            }
        }
        false
    }
}

fn lookup_closing(c: char) -> Option<(char, BracketType)> {
    use bidi_brackets::BIDI_BRACKETS;
    if let Ok(idx) = BIDI_BRACKETS.binary_search_by_key(&c, |&(left, _, _)| left) {
        let entry = &BIDI_BRACKETS[idx];
        return Some((entry.1, entry.2));
    }
    None
}

pub fn bidi_class_for_char(c: char) -> BidiClass {
    use core::cmp::Ordering;
    if let Ok(idx) = bidi_class::BIDI_CLASS.binary_search_by(|&(lower, upper, _)| {
        if c >= lower && c <= upper {
            Ordering::Equal
        } else if c < lower {
            Ordering::Greater
        } else if c > upper {
            Ordering::Less
        } else {
            unreachable!()
        }
    }) {
        let entry = &bidi_class::BIDI_CLASS[idx];
        if c >= entry.0 && c <= entry.1 {
            return entry.2;
        }
    }
    // extracted/DerivedBidiClass.txt says:
    // All code points not explicitly listed for Bidi_Class
    //  have the value Left_To_Right (L).
    BidiClass::LeftToRight
}
