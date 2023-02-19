use pest::error::{ErrorVariant, InputLocation};
use pest::iterators::Pair;
use std::fmt::{self, Display};
use std::str;
use thiserror::Error;

use super::Rule;

/// An entry of a post-parsing error, logged by [`ErrorLogger`].
#[derive(Debug)]
pub struct ErrorEntry {
    /// The beginning and end of the offending code.
    pub span: (usize, usize),
    /// The error message for this error.
    pub error: String,
}

impl From<pest::error::Error<Rule>> for ErrorEntry {
    fn from(value: pest::error::Error<Rule>) -> Self {
        let span = match value.location {
            InputLocation::Pos(pos) => (pos, pos + 1),
            InputLocation::Span((start, end)) => (start, end),
        };
        let error = match value.variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                let mut message = String::new();
                let or_list = |v: &[Rule]| match v {
                    [] => unreachable!(),
                    [r0] => format!("{}", r0.name()),
                    [r0, r1] => format!("{} or {}", r0.name(), r1.name()),
                    [r0, r1, r2] => format!("{}, {} or {}", r0.name(), r1.name(), r2.name()),
                    [r0, r1, r2, tail @ ..] => {
                        format!(
                            "{}, {}, {} or {} more possibilities",
                            r0.name(),
                            r1.name(),
                            r2.name(),
                            tail.len()
                        )
                    }
                };

                if negatives.len() > 0 {
                    message.push_str(&format!("Found {}.", or_list(&negatives)));
                }

                if positives.len() > 0 {
                    message.push_str(&format!("Expected {}.", or_list(&positives)));
                }

                message
            }
            ErrorVariant::CustomError { message } => message,
        };

        dbg!(ErrorEntry { span, error })
    }
}

impl ErrorEntry {
    /// Creates a human-readable form for this error entry, given the input it was derived from.
    pub(super) fn to_string_with(&self, input: &str) -> String {
        let (line_start, col_start) = dbg!(crate::utils::line_col(input, self.span.0));
        let (line_end, col_end) = dbg!(crate::utils::line_col(input, self.span.1));

        // The string buffer for this error message.
        let mut string = String::new();

        // The header indicating where the error starts.
        string.push_str(&format!(
            "Starting at line {}, col {}\n",
            line_start + 1,
            col_start + 1
        ));

        // The size of the margin to be set to fit the line number.
        let line_display_gap: String = std::iter::repeat(' ')
            .take((line_end + 1).to_string().len())
            .collect();

        // Start with an empty line:
        string.push_str(&format!(" {line_display_gap} \u{007c}\n"));

        // For each line in which the error appears, do:
        for (i, line) in input
            .lines()
            .enumerate()
            .skip(line_start)
            .take(line_end - line_start + 1)
        {
            // Print the line:
            string.push_str(&format!(" {} \u{007c} {line}\n", i + 1));

            // Now, underline the error portion...

            // Get the starting and ending point of the error:
            let start_point = if line_start != line_end && i != line_start {
                0
            } else {
                col_start
            };
            let end_point = if line_start != line_end && i != line_end {
                line.chars().count()
            } else {
                col_end
            };

            // Print the error line point:
            string.push_str(&format!(" {line_display_gap} \u{007c} "));
            for _ in 0..start_point {
                string.push(' ');
            }
            for _ in 0..(end_point - start_point) {
                string.push('^');
            }
            string.push('\n');
        }

        // End with an empty line:
        string.push_str(&format!(" {line_display_gap} \u{007c}\n"));

        // Print the error message itself.
        string.push_str(&format!(" {line_display_gap} = {}", self.error));

        string
    }
}

/// A logger of errors that happen post-parsing. Post parsing always succeeds, even with
/// a list of errors. It's the whole parsing processing that fails if there are
/// post-parsing errors.
#[derive(Debug)]
pub struct ErrorLogger<'a> {
    input: &'a str,
    /// The list of errors found during post-parsing, in the orders they were found.
    pub errors: Vec<ErrorEntry>,
}

impl ErrorLogger<'_> {
    pub(super) fn new(input: &str) -> ErrorLogger {
        ErrorLogger {
            input,
            errors: vec![],
        }
    }

    /// "Absorbs" an error.
    pub(super) fn absorb<T, E>(&mut self, pair: &Pair<Rule>, r: Result<T, E>) -> T
    where
        T: Default,
        E: ToString,
    {
        match r {
            Ok(ok) => ok,
            Err(err) => {
                self.errors.push(ErrorEntry {
                    span: (pair.as_span().start(), pair.as_span().end()),
                    error: err.to_string(),
                });
                T::default()
            }
        }
    }
}

/// A general parsing error.
#[derive(Debug, Error)]
pub struct ParseError {
    pub(super) errors: Vec<String>,
}

impl From<ErrorLogger<'_>> for ParseError {
    fn from(value: ErrorLogger<'_>) -> Self {
        ParseError {
            errors: value
                .errors
                .into_iter()
                .map(|entry| entry.to_string_with(value.input))
                .collect(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            write!(f, "\n{error}")?;
        }

        Ok(())
    }
}
