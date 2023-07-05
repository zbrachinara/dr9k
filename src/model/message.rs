use std::{iter::FlatMap, str::Split};

use itertools::Itertools;
use linkify::Span;

#[derive(PartialEq, Eq, Debug)]
pub struct Message {
    units: Vec<Unit>,
}

#[derive(PartialEq, Eq, Debug)]
enum Unit {
    Word(String),
    Link(String),
}

enum ParseLinkSpans {
    Link(String),
    Words(ParseWordsOwned),
}

impl<'a> From<Span<'a>> for ParseLinkSpans {
    fn from(value: Span) -> Self {
        let string = value.as_str().to_string();
        if value.kind().is_some() {
            Self::Link(string)
        } else {
            Self::Words(string.into())
        }
    }
}

impl Iterator for ParseLinkSpans {
    type Item = Unit;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Link(s) => (!s.is_empty()).then(|| Unit::Link(std::mem::take(s))),
            Self::Words(w) => w.next(),
        }
    }
}

self_cell::self_cell! {
    struct ParseWordsOwned {
        owner: String,
        #[not_covariant]
        dependent: ParseWords,
    }
}

impl From<String> for ParseWordsOwned {
    fn from(string: String) -> Self {
        fn pattern(c: char) -> bool {
            (c.is_ascii_punctuation() && c != '\'') || c.is_whitespace()
        }

        fn process_word(s: &str) -> Option<String> {
            if s.is_empty() {
                None
            } else {
                Some(
                    s.chars()
                        .filter(|&c| c != '\'')
                        .flat_map(char::to_lowercase)
                        .collect::<String>(),
                )
            }
        }

        Self::new(string, |string| {
            ParseWords(
                string
                    .split(pattern as FilterFn)
                    .flat_map(process_word as FlattenFn),
            )
        })
    }
}

type FilterFn = fn(char) -> bool;
type FlattenFn = fn(&str) -> Option<String>;
struct ParseWords<'a>(FlatMap<Split<'a, FilterFn>, Option<String>, FlattenFn>);

impl Iterator for ParseWordsOwned {
    type Item = Unit;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, de| de.0.next()).map(Unit::Word)
    }
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        // split string by links
        let split_by_link = linkify::LinkFinder::new()
            .url_must_have_scheme(true)
            .spans(s);

        let r = split_by_link.flat_map(ParseLinkSpans::from);

        Self {
            units: r.collect_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Message;

    macro_rules! w {
        ($s:expr) => {
            super::Unit::Word($s.to_string())
        };
    }

    #[test]
    fn basic() {
        assert_eq!(
            Message::from("a,    b,   c, and   d and    \t e").units,
            vec![
                w!("a"),
                w!("b"),
                w!("c"),
                w!("and"),
                w!("d"),
                w!("and"),
                w!("e")
            ]
        );
    }
}
