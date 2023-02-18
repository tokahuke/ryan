use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::rc::Rc;

use pest::iterators::Pairs;

use crate::environment::Environment;
use crate::rc_world;
use crate::utils::QuotedStr;

use super::value::Value;
use super::Context;
use super::ErrorLogger;
use super::Expression;
use super::Rule;
use super::State;

/// The way the imported value should be imported into Ryan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Import the content as text, verbatim. No evaluation is done on the imported
    /// content.
    Text,
    /// Import the value as a Ryan. This will execute the provided content as a Ryan
    /// program and will returning its output value.
    Ryan,
}

impl Format {
    pub(crate) fn load(
        self,
        env: Environment,
        mut reader: Box<dyn Read>,
    ) -> Result<Value, Box<dyn Error + 'static>> {
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        match self {
            Self::Text => Ok(Value::Text(rc_world::string_to_rc(text))),
            Self::Ryan => {
                let parsed = crate::parser::parse(&text).map_err(Box::new)?;
                let value = crate::parser::eval(env.clone(), &parsed).map_err(Box::new)?;

                Ok(value)
            }
        }
    }
}

/// An import statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// The path from which the content will be imported.
    pub path: Rc<str>,
    /// The way to interpret the imported content.
    pub format: Format,
    /// A default value in case the value cannot be imported.
    pub default: Option<Box<Expression>>,
}

impl Display for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            Format::Ryan => write!(f, "import {}", QuotedStr(&self.path))?,
            Format::Text => write!(f, "import {} as text", QuotedStr(&self.path))?,
        }

        if let Some(default) = &self.default {
            write!(f, " or {default}")?;
        }

        Ok(())
    }
}

impl Import {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut path = None;
        let mut format = None;
        let mut default = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::text => {
                    path = Some(rc_world::string_to_rc(
                        logger.absorb(&pair, crate::utils::unescape(pair.as_str())),
                    ))
                }
                Rule::importFormatText => format = Some(Format::Text),
                Rule::expression => default = Some(Expression::parse(logger, pair.into_inner())),
                _ => unreachable!(),
            }
        }

        Import {
            path: path.expect("there is always a path in an import"),
            format: format.unwrap_or(Format::Ryan),
            default: default.map(Box::new),
        }
    }

    pub(super) fn eval(&self, state: &mut State) -> Option<Value> {
        state.push_ctx(Context::LoadingImport(self.path.clone()));

        let value = match state.environment.load(self.format, &self.path) {
            Ok(value) => value,
            Err(err) => {
                if let Some(default) = &self.default {
                    default.eval(state)?
                } else {
                    state.absorb(Err(err))?
                }
            }
        };

        state.pop_ctx();

        Some(value)
    }
}
