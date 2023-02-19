use std::fmt::Display;

use std::fmt;

use thiserror::Error;

pub fn fmt_list<I>(f: &mut fmt::Formatter<'_>, it: I) -> fmt::Result
where
    I: IntoIterator,
    I::Item: Display,
{
    let mut it = it.into_iter();

    if let Some(item) = it.next() {
        write!(f, "{item}")?;
    }

    while let Some(item) = it.next() {
        write!(f, ", {item}")?;
    }

    Ok(())
}

pub fn fmt_map<I, K, V>(f: &mut fmt::Formatter<'_>, it: I) -> fmt::Result
where
    I: IntoIterator<Item = (K, V)>,
    K: Display,
    V: Display,
{
    let mut it = it.into_iter();

    if let Some((key, value)) = it.next() {
        write!(f, "{key}: {value}")?;
    }

    while let Some((key, value)) = it.next() {
        write!(f, ", {key}: {value}")?;
    }

    Ok(())
}

/// A string displayed as quoted as per the JSON string rules.
pub(crate) struct QuotedStr<'a>(pub &'a str);

impl QuotedStr<'_> {
    pub fn quote(&self) -> String {
        let mut string = String::with_capacity(self.0.len() + 2);
        string.push('"');

        for ch in self.0.chars() {
            match ch {
                '"' => string.push_str(r#"\""#),
                '\\' => string.push_str(r"\\"),
                '\u{0008}' => string.push_str(r"\b"),
                '\u{000c}' => string.push_str(r"\f"),
                '\n' => string.push_str(r"\n"),
                '\r' => string.push_str(r"\r"),
                '\t' => string.push_str(r"\t"),
                ch => string.push(ch),
            }
        }

        string.push('"');

        string
    }
}

impl Display for QuotedStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.quote())
    }
}

#[derive(Debug, Error)]
pub enum UnescapeError {
    #[error("Missing starting double quote in escaped string")]
    NoStartingQuote,
    #[error("No such escape sequence \\{0}")]
    UnknownEscape(char),
    #[error("Expected hexadecimal digit, got {0:?}")]
    NotADigit(char),
    #[error("The character \\u{0:x} is not valid unicode")]
    NotUnicode(u32),
    #[error("Quoted string ended before the end of the input")]
    SpuriousTail,
    #[error("Input ended before the ending double quote in escaped string")]
    NoEndingQuote,
}

/// Unquotes a string, as per the official JSON rules.
///
/// See https://stackoverflow.com/questions/19176024/ for implementation.
pub(crate) fn unescape(s: &str) -> Result<String, UnescapeError> {
    let mut chars = s.chars();
    let mut next = move || chars.next().ok_or(UnescapeError::NoEndingQuote);
    let mut string = String::with_capacity(s.len());

    if next()? != '"' {
        return Err(UnescapeError::NoStartingQuote);
    }

    loop {
        match next()? {
            '"' => break,
            '\\' => match next()? {
                '"' => string.push('"'),
                '\\' => string.push('\\'),
                '/' => string.push('/'),
                'b' => string.push('\u{0008}'),
                'f' => string.push('\u{000c}'),
                'n' => string.push('\n'),
                'r' => string.push('\r'),
                't' => string.push('\t'),
                'u' => {
                    // This could be a closure, but debugging got the best of me...
                    macro_rules! next_digit {
                        () => {
                            next().and_then(|ch| {
                                ch.to_digit(16).ok_or(UnescapeError::NotADigit(ch))
                            })?
                        };
                    }
                    // Descending order...
                    let code = (next_digit!() << 12)
                        + (next_digit!() << 8)
                        + (next_digit!() << 4)
                        + (next_digit!() << 0);
                    let ch = char::from_u32(code).ok_or(UnescapeError::NotUnicode(code))?;
                    string.push(ch);
                }
                unknown => return Err(UnescapeError::UnknownEscape(unknown)),
            },
            ch => string.push(ch),
        }
    }

    // If error, the whole input has been consumed and everything is ok.
    if next().is_err() {
        Ok(string)
    } else {
        Err(UnescapeError::SpuriousTail)
    }
}

pub(crate) fn line_col(input: &str, idx: usize) -> (usize, usize) {
    let mut lines = 0;
    let mut pos = 0;

    for ch in input.chars().take(idx) {
        if ch == '\n' {
            lines += 1;
            pos = 0;
        } else {
            pos += 1;
        }
    }

    (lines, pos)
}
