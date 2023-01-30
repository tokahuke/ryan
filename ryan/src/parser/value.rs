use std::fmt::Display;
use std::{collections::HashMap, rc::Rc};

use thiserror::Error;

use crate::environment::NativePatternMatch;
use crate::utils::QuotedStr;

use super::block::Block;
use super::literal::Literal;
use super::pattern::{BindError, Pattern};
use super::types::Type;
use super::{Context, State};

/// A pattern match rule introduced by a biding.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternMatch {
    /// The pattern agains which the input will be matched.
    pub pattern: Pattern,
    /// The block to be executes if the match is successful.
    pub block: Block,
    /// The variable from the program necessary for the block to evaluate correctly.
    pub captures: HashMap<Rc<str>, Value>,
}

impl Display for PatternMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: make the synxax for this representation:

        // if let Some(id) = &self.identifier {
        //     write!(f, "@{id} ")?;
        // }

        write!(f, "{} let {{", self.pattern)?;
        crate::utils::fmt_map(f, self.captures.iter().map(|(k, v)| (QuotedStr(k), v)))?;
        write!(f, "}} => {}", self.block)?;

        Ok(())
    }
}

impl PatternMatch {
    pub(super) fn r#match(
        &self,
        arg: &Value,
        state: &mut State<'_>,
    ) -> Option<Result<Value, BindError>> {
        let mut new_bindings = self.captures.clone();

        if let Err(err) = self.pattern.bind(&arg, &mut new_bindings, state)? {
            return Some(Err(err));
        }

        let mut new_state = State {
            inherited: Some(&state),
            bindings: new_bindings,
            error: None,
            contexts: vec![],
            environment: state.environment.clone(),
        };

        let outcome = self.block.eval(&mut new_state);
        let maybe_error = new_state.error;

        if let Some(outcome) = outcome {
            Some(Ok(outcome))
        } else {
            state.contexts.extend(new_state.contexts);
            state.error = maybe_error;
            return None;
        }
    }
}

impl NativePatternMatch {
    pub(super) fn r#match(&self, arg: Value, state: &mut State<'_>) -> Option<Value> {
        state.push_ctx(Context::SubstitutingPattern(Some(self.identifier.clone())));
        let value = state.absorb((self.func)(arg))?;
        state.pop_ctx();

        Some(value)
    }
}

/// An error raised when a [`Value`] has no counterpart in JSON, e.g., a type or a pattern
/// match rule.
#[derive(Debug, Error)]
#[error("The follosing value is not JSON-serializable: {value}")]
pub struct NotRepresentable {
    value: String,
}

/// A Ryan value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The value `null`.
    Null,
    /// A boolean.
    Bool(bool),
    /// An integer.
    Integer(i64),
    /// A floating point, including scarry stuff like `inf` and `NaN`.
    Float(f64),
    /// An utf-8 encoded string.
    Text(Rc<str>),
    /// A list of other Ryan values.
    List(Rc<[Value]>),
    /// An association of strings to other Ryan values.
    Map(Rc<HashMap<Rc<str>, Value>>),
    /// A list of pattern match rules for a given identifier.
    PatternMatches(Rc<str>, Vec<Rc<PatternMatch>>),
    /// A pattern match where the code to be executed in case of a match is navtive code,
    /// not a Ryan block.
    NativePatternMatch(Rc<NativePatternMatch>),
    /// A Ryan type.
    Type(Type),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null")?,
            Self::Bool(b) => write!(f, "{b}")?,
            Self::Integer(int) => write!(f, "{int}")?,
            Self::Float(float) => write!(f, "{float}")?,
            Self::Text(text) => write!(f, "{text:?}")?,
            Self::List(list) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, list.iter())?;
                write!(f, "]")?;
            }
            Self::Map(map) => {
                write!(f, "{{")?;
                crate::utils::fmt_map(f, map.iter())?;
                write!(f, "}}")?;
            }
            Self::PatternMatches(name, pattern_matches) => {
                if pattern_matches.len() > 1 {
                    write!(
                        f,
                        "![match {name} with {} alternatives]",
                        pattern_matches.len()
                    )?;
                } else {
                    write!(f, "![match {name} with 1 alternative]")?;
                }
            }
            Self::NativePatternMatch(pattern_match) => {
                write!(f, "{pattern_match}")?;
            }
            Self::Type(r#type) => write!(f, "{type}")?,
        };

        Ok(())
    }
}

impl Value {
    /// Tests the "truthiness" of a value. Currently, only `true` is true; values other
    /// than a boolen will raise an error.
    pub fn is_true(&self) -> Result<bool, String> {
        match self {
            Self::Bool(b) => Ok(*b),
            anything_else => Err(format!("Value `{anything_else}` is not a boolean")),
        }
    }

    /// "Equality" between a value and a [`Literal`]. Litterals are nodes in the abstract
    /// syntax tree, while values are not.
    pub fn matches(&self, lit: &Literal) -> bool {
        match (self, lit) {
            (Value::Integer(val), Literal::Integer(lit)) if val == lit => true,
            (Value::Float(val), Literal::Float(lit)) if val == lit => true,
            (Value::Bool(val), Literal::Bool(lit)) if val == lit => true,
            (Value::Text(val), Literal::Text(lit)) if val.as_ref() == lit => true,
            _ => false,
        }
    }

    fn extract_item(&self, item: &Value) -> Result<Value, String> {
        match (self, item) {
            (Value::Map(map), Value::Text(key)) => {
                if let Some(value) = map.get(key) {
                    Ok(value.clone())
                } else {
                    Err(format!("Key {key:?} missing in map"))
                }
            }
            (Value::List(list), Value::Integer(idx)) => {
                let idx = *idx as usize;
                if idx < list.len() {
                    Ok(list[idx].clone())
                } else {
                    Err(format!(
                        "Tried to access index {idx} of list of length {}",
                        list.len()
                    ))
                }
            }
            (val, item) => Err(format!("Cannot index {val} by {item}")),
        }
    }

    /// Extracts the value lying at the end of a path in a nested Ryan value.
    pub fn extract_path(&self, path: &[Value]) -> Result<Value, String> {
        match (self, path) {
            (val, []) => Ok(val.clone()),
            (val, [item, tail @ ..]) => val
                .extract_item(item)
                .and_then(|extracted| extracted.extract_path(tail)),
        }
    }

    /// Creates the equivalent JSON representation of the current Ryan value, if possible.
    pub fn to_json(&self) -> Result<serde_json::Value, NotRepresentable> {
        let json = match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(bool) => serde_json::Value::Bool(*bool),
            Self::Integer(int) => serde_json::Value::Number((*int).into()),
            Self::Float(float) => serde_json::Value::Number(
                serde_json::Number::from_f64(*float).ok_or_else(|| NotRepresentable {
                    value: Self::Float(*float).to_string(),
                })?,
            ),
            Self::Text(text) => serde_json::Value::String(text.to_string()),
            Self::List(list) => {
                serde_json::Value::Array(list.iter().map(Self::to_json).collect::<Result<_, _>>()?)
            }
            Self::Map(map) => serde_json::Value::Object(
                map.iter()
                    .map(|(key, value)| value.to_json().map(|v| (key.to_string(), v)))
                    .collect::<Result<_, _>>()?,
            ),
            bad => {
                return Err(NotRepresentable {
                    value: bad.clone().to_string(),
                })
            }
        };

        Ok(json)
    }
}
