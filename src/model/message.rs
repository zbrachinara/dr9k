use std::str::FromStr;

use itertools::Itertools;
use linkify::Span;

#[derive(PartialEq, Eq)]
pub struct Message {
    units: Vec<Unit>,
}

#[derive(PartialEq, Eq)]
enum Unit {
    Word(String),
    Link(String),
}

enum ParseLinkSpans {
    Link(String),
    Words(ParseWords),
}

impl<'a> From<Span<'a>> for ParseLinkSpans {
    fn from(value: Span) -> Self {
        let string = value.as_str().to_string();
        if value.kind().is_some() {
            Self::Link(string)
        } else {
            Self::Words(ParseWords(string))
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

struct ParseWords(String);

impl Iterator for ParseWords {
    type Item = Unit;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.split_once(|c: char| c.is_whitespace());

        todo!()
    }
}

impl FromStr for Message {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split string by links
        let split_by_link = linkify::LinkFinder::new()
            .url_must_have_scheme(true)
            .spans(s);

        let r = split_by_link.flat_map(ParseLinkSpans::from);

        Ok(Self {
            units: r.collect_vec(),
        })
    }
}
