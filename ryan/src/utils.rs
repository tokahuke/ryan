use std::fmt::Display;

use std::fmt;

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

pub struct QuotedStr<'a>(pub &'a str);

impl Display for QuotedStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
