use pest::iterators::Pairs;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::rc_world;

use super::value::Value;
use super::ErrorLogger;
use super::Rule;
use super::State;

/// A literal Ryan value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// The value `null`.
    Null,
    /// An integer.
    Integer(i64),
    /// A float.
    Float(f64),
    /// A boolean.
    Bool(bool),
    /// An utf-8 encoded string.
    Text(String),
    /// An identifier, i.e., the name of a variable, a type or a pattern.
    Identifier(Rc<str>),
}

impl Default for Literal {
    fn default() -> Self {
        Literal::Integer(0)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Integer(int) => write!(f, "{int}"),
            Self::Float(float) => write!(f, "{float}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Text(text) => write!(f, "{text:?}"),
            Self::Identifier(id) => write!(f, "{id}"),
        }
    }
}

impl Literal {
    pub(super) fn parse(logger: &mut ErrorLogger, mut pairs: Pairs<'_, Rule>) -> Self {
        let pair = pairs.next().expect("there is always a token in a literal");

        let literal = match pair.as_rule() {
            Rule::null => Literal::Null,
            Rule::number => logger.absorb(
                &pair,
                pair.as_str()
                    .parse::<i64>()
                    .map(|int| Literal::Integer(int))
                    .or_else(|_| {
                        pair.as_str()
                            .parse::<f64>()
                            .map(|float| Literal::Float(float))
                    }),
            ),
            Rule::bool => match pair.as_str() {
                "true" => Literal::Bool(true),
                "false" => Literal::Bool(false),
                _ => unreachable!(),
            },
            Rule::text => Literal::Text(logger.absorb(&pair, snailquote::unescape(pair.as_str()))),
            Rule::identifier => Literal::Identifier(rc_world::str_to_rc(pair.as_str())),
            _ => unreachable!(),
        };

        literal
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &[Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        if let Self::Identifier(id) = self {
            match state.try_get(id) {
                Ok(cap) => {
                    values.insert(id.clone(), cap.clone());
                }
                Err(err) => {
                    if !provided.contains(&id) {
                        state.absorb(Err(err))?;
                    }
                }
            }
        }

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        let value = match self {
            Self::Null => Value::Null,
            Self::Bool(b) => Value::Bool(*b),
            Self::Integer(int) => Value::Integer(*int),
            Self::Float(float) => Value::Float(*float),
            Self::Text(text) => Value::Text(rc_world::str_to_rc(&text)),
            Self::Identifier(id) => state.get(id)?,
        };

        Some(value)
    }
}
