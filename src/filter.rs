//! What typing in a filter field *means*.
//!
//! A funnel or a magnifier, one line of text, and a list to narrow: that field is
//! specified here already — [`TextField`](crate::components::TextField) with a filter
//! prefix, [`SearchBox`](crate::components::SearchBox), the needle in a
//! [`ComboBox`](crate::components::ComboBox). This is the other half of the same
//! component, because what the text does is as much a part of it as what it looks like,
//! and a filter that means one thing in one window and something else in the next is the
//! same kind of inconsistency as two greys.
//!
//! [`Query::parse`] turns a line of typing into something you can ask of a candidate;
//! [`Query::matches`] asks it. The whole syntax:
//!
//! | typed | keeps |
//! | --- | --- |
//! | `report` | anything containing `report`, in any case |
//! | `wor he` | anything containing **both** words, in **either order** |
//! | `!tmp` | anything *not* containing `tmp` |
//! | `^src` | anything **beginning** with `src` |
//! | `.rs$` | anything **ending** with `.rs` |
//! | `^a$` | exactly `a` |
//! | `!^~` | anything not beginning with `~` |
//!
//! ```
//! use azur_egui_theme::filter::Query;
//!
//! let q = Query::parse("wor he");
//! assert!(q.matches("hello world"));
//! assert!(q.matches("world hello"));
//! assert!(!q.matches("hello to you"));
//! ```
//!
//! # Words in any order is the whole point
//!
//! One substring is a poor way to find something you half remember. You know two things
//! about it and not the order they appear in — a project and an extension, a machine and
//! a date, a surname and a street. Every word narrowing independently turns those two
//! half-memories into one query, and it is why this is worth being a syntax at all
//! rather than a `contains`.
//!
//! # No regex engine, and not because of the dependency
//!
//! "Every word, any order" is written `(?=.*a)(?=.*b).*` as a regex, and that shape is
//! the reason not to use one. The [`regex`](https://docs.rs/regex) crate **cannot
//! express it**: it has no lookahead, by design, because lookahead is what costs it the
//! linear-time guarantee. Expressing it needs a backtracking engine, and a backtracking
//! engine is exactly what a filter field must not have — the text comes from a person
//! typing, one candidate at a time, over a list that can be hundreds of thousands long.
//!
//! And there is nothing left to express. A chain of lookaheads over literals says "each
//! of these appears somewhere", which is a loop over the words and a substring test. The
//! anchors say "at this end" and the negation says "nowhere", which are the same test
//! twice more. So the engine buys nothing here, and what it would cost is a compile per
//! keystroke plus a backtrack per candidate.
//!
//! Which also means **the words are literals**. `a.c` finds `a.c` and not `abc`; a
//! filter field is somewhere you type a fragment of a name, and a name is full of `.`,
//! `(`, `+` and `-`.
//!
//! # Case
//!
//! Case-insensitive, and folded once at [`Query::parse`] rather than per candidate.
//! Beyond ASCII it folds per [`char`] — `Élément` is found by `élé` — which is a
//! slower path taken only when one side is not ASCII. Two cased letters do not survive
//! it: `ß`/`SS` and Turkish dotless `ı` change length when lowercased, and both compare
//! unequal here rather than wrongly equal.

/// Where a word has to be, rather than merely somewhere.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Anchor {
    Anywhere,
    Start,
    End,
    /// `^word$`: the candidate is the word and nothing else.
    Whole,
}

/// One word of a query, folded and with its markers taken off.
struct Term {
    /// Lowercased here, once, instead of per candidate. Never empty: a word that is
    /// nothing but markers is dropped at parse time.
    word: String,
    anchor: Anchor,
    /// `!`: the term holds when the word is *not* there.
    negated: bool,
    /// Whether `word` is all ASCII, which decides whether a candidate can take the byte
    /// path. Answered here rather than per candidate.
    ascii: bool,
}

impl Term {
    fn new(word: &str, anchor: Anchor, negated: bool) -> Self {
        let word: String = word.chars().map(fold).collect();
        Self {
            ascii: word.is_ascii(),
            word,
            anchor,
            negated,
        }
    }

    /// Whether `hay` satisfies this one term.
    fn holds(&self, hay: &str) -> bool {
        // Both sides ASCII is the overwhelming common case and worth its own path: bytes
        // compare without decoding, and `is_ascii` is a straight scan of memory.
        let found = if self.ascii && hay.is_ascii() {
            in_bytes(hay.as_bytes(), self.word.as_bytes(), self.anchor)
        } else {
            in_chars(hay, &self.word, self.anchor)
        };
        found != self.negated
    }
}

/// A parsed filter field: the words, their anchors, and which of them are exclusions.
///
/// Parse once and keep it for as long as the text is unchanged — over a list of any
/// size, parsing per candidate would be the expensive half of the work.
///
/// ```
/// use azur_egui_theme::filter::Query;
///
/// let q = Query::parse("!test .rs$");
/// assert!(q.matches(r"src\lib.rs"));
/// assert!(!q.matches(r"src\lib_test.rs")); // excluded by `!test`
/// assert!(!q.matches(r"src\lib.rs.bak")); // `.rs` is not at the end
/// ```
#[derive(Default)]
pub struct Query {
    terms: Vec<Term>,
}

impl Query {
    /// Read a line of typing.
    ///
    /// Split on whitespace, so empty stretches and stray indentation ask nothing.
    /// Leading `!`, then leading `^`, then trailing `$`: in that order, so `!^tmp` reads
    /// as "does not begin with `tmp`". **A doubled marker is the marker itself** — `!!`
    /// finds a literal `!`, which is the only way to filter for a name that starts with
    /// one, and `$$` a literal `$` at the end.
    ///
    /// Never fails. There is no such thing as a filter field holding a syntax error:
    /// every line of text has to mean something while it is half typed.
    pub fn parse(text: &str) -> Self {
        let mut terms = Vec::new();
        for word in text.split_whitespace() {
            let (negated, word) = marker(word, '!');
            let (start, word) = marker(word, '^');
            let (end, word) = end_marker(word);
            // `!`, `^`, `$`, `^$` on their own: a marker with no word is not a question.
            // Dropping them is what lets a query be typed left to right — `^` then `s`
            // then `r` narrows all the way down without ever matching nothing.
            if word.is_empty() {
                continue;
            }
            let anchor = match (start, end) {
                (true, true) => Anchor::Whole,
                (true, false) => Anchor::Start,
                (false, true) => Anchor::End,
                (false, false) => Anchor::Anywhere,
            };
            terms.push(Term::new(word, anchor, negated));
        }
        Self { terms }
    }

    /// Whether this query asks nothing, and so keeps everything.
    ///
    /// [`matches`](Self::matches) already answers `true` for every candidate in that
    /// case; this is for the caller that would rather not walk the list at all.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Whether `text` survives every word of the query.
    pub fn matches(&self, text: &str) -> bool {
        self.terms.iter().all(|term| term.holds(text))
    }
}

/// One-shot: parse `filter` and ask it of `text`.
///
/// For a single candidate — a title, a path, one row being tested for one reason. Over a
/// list, hold a [`Query`] instead, or the parse is paid per row.
pub fn matches(text: &str, filter: &str) -> bool {
    Query::parse(filter).matches(text)
}

/// The syntax, short enough to put in a tooltip beside the field.
///
/// Here rather than in each application so that the field means the same thing
/// everywhere *and says so* in the same words. Two lines, no columns to align — the
/// tooltip is set in the caption face, which is not monospaced.
pub const SYNTAX: &str = "Every word must match, in any order.\n\
                          !word excludes it, ^word matches at the start, word$ at the end.";

/// Take a leading marker off, unless it is doubled — in which case take one off and
/// leave the marker as part of the word.
fn marker(word: &str, mark: char) -> (bool, &str) {
    match word.strip_prefix(mark) {
        Some(rest) if rest.starts_with(mark) => (false, rest),
        Some(rest) => (true, rest),
        None => (false, word),
    }
}

/// The same, for the trailing `$`.
fn end_marker(word: &str) -> (bool, &str) {
    match word.strip_suffix('$') {
        Some(rest) if rest.ends_with('$') => (false, rest),
        Some(rest) => (true, rest),
        None => (false, word),
    }
}

/// Fold one character for comparison.
///
/// ASCII directly; otherwise the first character of its Unicode lowercase, which is the
/// whole of it for every letter that lowercases to one character. That restriction is
/// what keeps the two sides walkable in step: folding never changes how many characters
/// there are, so a word of *n* characters is compared against exactly *n* of a
/// candidate's, and `ß`/`SS` simply fails to match rather than matching at the wrong
/// length.
#[inline]
fn fold(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

/// `word` is already folded; `hay` is folded a byte at a time as it is read.
fn in_bytes(hay: &[u8], word: &[u8], anchor: Anchor) -> bool {
    let (n, m) = (hay.len(), word.len());
    if m > n {
        return false;
    }
    let same = |at: usize| {
        hay[at..at + m]
            .iter()
            .zip(word)
            .all(|(h, w)| h.to_ascii_lowercase() == *w)
    };
    match anchor {
        Anchor::Start => same(0),
        Anchor::End => same(n - m),
        Anchor::Whole => n == m && same(0),
        // Naive, with the first byte checked before the rest is worth touching. Over a
        // name or a path — a few hundred bytes against a word somebody typed — the
        // memchr-shaped win is smaller than the dependency.
        Anchor::Anywhere => {
            let first = word[0];
            (0..=n - m).any(|at| hay[at].to_ascii_lowercase() == first && same(at))
        }
    }
}

/// The same four questions where one side is not ASCII, walked as characters.
fn in_chars(hay: &str, word: &str, anchor: Anchor) -> bool {
    match anchor {
        Anchor::Start => begins(hay, word),
        Anchor::Whole => equal(hay, word),
        Anchor::Anywhere => hay.char_indices().any(|(at, _)| begins(&hay[at..], word)),
        // The last `k` characters, where `k` is the word's — reached from the back, so a
        // long candidate is not walked to find its own tail.
        Anchor::End => match hay.char_indices().nth_back(word.chars().count() - 1) {
            Some((at, _)) => equal(&hay[at..], word),
            None => false,
        },
    }
}

/// Whether `hay` begins with the already-folded `word`.
fn begins(hay: &str, word: &str) -> bool {
    let mut hay = hay.chars();
    word.chars()
        .all(|w| matches!(hay.next(), Some(h) if fold(h) == w))
}

/// Whether `hay` *is* the already-folded `word`.
fn equal(hay: &str, word: &str) -> bool {
    let (mut hay, mut word) = (hay.chars(), word.chars());
    loop {
        match (hay.next(), word.next()) {
            (None, None) => return true,
            (Some(h), Some(w)) if fold(h) == w => {}
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_word_is_a_case_insensitive_substring() {
        let q = Query::parse("  ReadMe ");
        assert!(q.matches("README.md"));
        assert!(q.matches("some-readme-file"));
        assert!(!q.matches("read.me"), "not a subsequence, a substring");
    }

    #[test]
    fn a_word_is_a_literal_and_not_a_pattern() {
        // The one thing a regex engine would have changed about the common case.
        let q = Query::parse("a.c");
        assert!(q.matches("xa.cy"));
        assert!(!q.matches("abc"));
        assert!(Query::parse("c++").matches("main.c++"));
    }

    #[test]
    fn every_word_must_match_in_any_order() {
        let q = Query::parse("wor he");
        assert!(q.matches("hello world"));
        assert!(q.matches("world hello"));
        assert!(!q.matches("hello to you"));
    }

    #[test]
    fn a_leading_bang_excludes() {
        let q = Query::parse("!tmp");
        assert!(q.matches("notes.txt"));
        assert!(!q.matches("tmp/notes.txt"));

        // And mixes with ordinary words, which is what it is for.
        let q = Query::parse("notes !tmp");
        assert!(q.matches("notes.txt"));
        assert!(!q.matches("tmp/notes.txt"));
        assert!(!q.matches("other.txt"));
    }

    #[test]
    fn a_caret_anchors_to_the_start_and_a_dollar_to_the_end() {
        let q = Query::parse("^src");
        assert!(q.matches("src/lib.rs"));
        assert!(!q.matches("mysrc/lib.rs"));

        let q = Query::parse(".rs$");
        assert!(q.matches("src/lib.rs"));
        assert!(!q.matches("src/lib.rs.bak"));

        // Exactly, when both ends are held.
        let q = Query::parse("^main.rs$");
        assert!(q.matches("MAIN.rs"));
        assert!(!q.matches("main.rs~"));
        assert!(!q.matches("xmain.rs"));
    }

    #[test]
    fn the_markers_compose() {
        let q = Query::parse("!^~");
        assert!(q.matches("report.docx"));
        assert!(!q.matches("~report.docx"), "does not begin with a tilde");

        let q = Query::parse("!.bak$ ^src");
        assert!(q.matches("src/lib.rs"));
        assert!(!q.matches("src/lib.rs.bak"));
        assert!(!q.matches("tests/lib.rs"));
    }

    #[test]
    fn a_marker_only_counts_at_its_own_end() {
        // A `^` in the middle and a `$` at the start are just characters, which is what
        // makes `$RECYCLE.BIN` and `a^b` findable at all.
        assert!(Query::parse("$RECYCLE").matches(r"C:\$Recycle.Bin"));
        assert!(Query::parse("a^b").matches("xa^by"));
        assert!(!Query::parse("a^b").matches("ab"));
    }

    #[test]
    fn a_doubled_marker_is_the_marker_itself() {
        let q = Query::parse("!!important");
        assert!(q.matches("!important.txt"), "a literal bang, not an escape");
        assert!(!q.matches("important.txt"));

        assert!(Query::parse("^^caret").matches("a ^caret here"));
        assert!(Query::parse("cost$$").matches("the cost$ of it"));
    }

    #[test]
    fn a_query_of_nothing_but_markers_asks_nothing() {
        // Every prefix of `^src` has to mean something, including the first keystroke.
        for text in ["^", "!", "$", "^$", "!^", " ! ^ $ "] {
            let q = Query::parse(text);
            assert!(q.is_empty(), "`{text}` should ask nothing");
            assert!(q.matches("anything"));
        }
    }

    #[test]
    fn an_empty_query_keeps_everything() {
        assert!(Query::parse("").is_empty());
        assert!(Query::parse("   \t ").is_empty());
        assert!(Query::parse("").matches(""));
        assert!(matches("anything", ""));
    }

    #[test]
    fn nothing_matches_inside_nothing() {
        let q = Query::parse("a ^b c$");
        assert!(!q.matches(""));
        // ...except an exclusion, which is satisfied by the absence.
        assert!(Query::parse("!a").matches(""));
    }

    #[test]
    fn case_folds_beyond_ascii() {
        let q = Query::parse("ÉLÉ");
        assert!(q.matches("un Élément"));
        assert!(q.matches("un élément"));

        // Both sides, either way round, and through the anchors too.
        assert!(Query::parse("élé").matches("ÉLÉMENT"));
        assert!(Query::parse("^élé").matches("Élément"));
        assert!(Query::parse("ément$").matches("Élément"));
        assert!(Query::parse("^élément$").matches("ÉLÉMENT"));
        assert!(!Query::parse("^lément$").matches("ÉLÉMENT"));
    }

    #[test]
    fn a_word_longer_than_the_candidate_cannot_be_in_it() {
        // The bounds arithmetic in both paths subtracts one length from the other, so a
        // word too long to fit has to be turned away before either subtraction. Both
        // paths, which is what the second row of words is for.
        for word in ["abcdef", "^abcdef", "abcdef$", "^abcdef$"] {
            assert!(!Query::parse(word).matches("abc"), "`{word}` in `abc`");
            assert!(!Query::parse(word).matches(""), "`{word}` in nothing");
        }
        for word in ["éàêîô", "^éàêîô", "éàêîô$"] {
            assert!(!Query::parse(word).matches("éàê"), "`{word}` in `éàê`");
            assert!(!Query::parse(word).matches(""), "`{word}` in nothing");
        }
        // Exactly as long is not too long.
        assert!(Query::parse("^abcdef$").matches("ABCDEF"));
        assert!(Query::parse("^éàêîô$").matches("ÉÀÊÎÔ"));
    }

    #[test]
    fn the_syntax_line_describes_every_marker() {
        // It is what the tooltip shows; a marker added without a word about it is a
        // feature nobody can find.
        for mark in ['!', '^', '$'] {
            assert!(SYNTAX.contains(mark), "`{mark}` is undocumented");
        }
    }
}
