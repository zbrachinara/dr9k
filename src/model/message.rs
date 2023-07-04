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

fn parse_word(s: &str) -> (Option<String>, &str) {
    if s.is_empty() {
        return (None, "");
    }

    let (split_by_whitespace, remainder) = s.split_once(char::is_whitespace).unwrap_or((s, ""));

    fn punct(c: char) -> bool {
        c.is_ascii_punctuation() && c != '\''
    }

    let trim_punct = split_by_whitespace.trim_start_matches(punct);
    let bare = trim_punct
        .split_once(punct)
        .map(|(w, _)| w)
        .unwrap_or(trim_punct);

    if bare.is_empty() {
        return parse_word(remainder);
    }

    let word = bare
        .chars()
        .filter(|&c| c != '\'')
        .flat_map(|c| c.to_lowercase())
        .collect::<String>();

    (Some(word), remainder)
}

impl Iterator for ParseWords {
    type Item = Unit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            let (word, remainder) = parse_word(self.0.as_str());
            let remainder = remainder.to_string();
            word.map(|u| {
                self.0 = remainder;
                Unit::Word(u)
            })
        }
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