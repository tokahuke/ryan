use std::cmp;
use std::fmt::Display;
use std::rc::Rc;

use indexmap::IndexMap;
use thiserror::Error;

use crate::environment::NativePatternMatch;
use crate::utils::QuotedStr;
use crate::DecodeError;

use super::block::Block;
use super::literal::Literal;
use super::pattern::{BindError, Pattern};
use super::types::Type;
use super::{Context, State};

/// A pattern match rule introduced by a biding.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternMatch {
    /// The pattern against which the input will be matched.
    pub pattern: Pattern,
    /// The block to be executes if the match is successful.
    pub block: Block,
    /// The variable from the program necessary for the block to evaluate correctly.
    pub captures: IndexMap<Rc<str>, Value>,
}

impl Display for PatternMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: make the syntax for this representation:

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
        state: &mut State,
    ) -> Option<Result<Value, BindError>> {
        let mut new_bindings = self.captures.clone();

        if let Err(err) = self.pattern.bind(&arg, &mut new_bindings, state)? {
            return Some(Err(err));
        }

        let mut new_state = state.new_local(new_bindings);
        let outcome = self.block.eval(&mut new_state)?;

        Some(Ok(outcome))
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
#[error("The following value is not JSON-serializable: {value}")]
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
    Map(Rc<IndexMap<Rc<str>, Value>>),
    /// A list of pattern match rules for a given identifier.
    PatternMatches(Rc<str>, Vec<Rc<PatternMatch>>),
    /// A pattern match where the code to be executed in case of a match is native code,
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
                write!(
                    f,
                    "![pattern {name} {}]",
                    pattern_matches
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(" | ")
                )?;
            }
            Self::NativePatternMatch(pattern_match) => {
                write!(f, "{pattern_match}")?;
            }
            Self::Type(r#type) => write!(f, "{type}")?,
        };

        Ok(())
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        let list = iter.into_iter().collect::<Vec<_>>();
        Value::List(list.into())
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let order = match (self, other) {
            (Self::Null, Self::Null) => cmp::Ordering::Equal,
            (Self::Bool(a), Self::Bool(b)) => a.cmp(b),
            (Self::Integer(a), Self::Integer(b)) => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b)?,
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            _ => return None,
        };

        Some(order)
    }
}

impl Value {
    /// Tests the "truthiness" of a value. Currently, only `true` is true; values other
    /// than a boolean will raise an error.
    pub fn is_true(&self) -> Result<bool, String> {
        match self {
            Self::Bool(b) => Ok(*b),
            anything_else => Err(format!("Value `{anything_else}` is not a boolean")),
        }
    }

    /// "Equality" between a value and a [`Literal`]. Literals are nodes in the abstract
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

    /// Does the indexing of a given value by another.
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

    /// Tries to return an iterator, if the value is iterable
    pub fn iter(&self) -> Result<ValueIter, NotIterable> {
        match self {
            Self::List(list) => Ok(ValueIter::List(list.iter())),
            Self::Map(dict) => Ok(ValueIter::Map(dict.iter())),
            _ => Err(NotIterable { val: self.clone() }),
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

    pub fn canonical_type(&self) -> Type {
        match self {
            Value::Null => Type::Null,
            Value::Bool(_) => Type::Bool,
            Value::Integer(_) => Type::Integer,
            Value::Float(_) => Type::Float,
            Value::Text(_) => Type::Text,
            Value::List(list) => {
                let types = list.iter().map(Value::canonical_type).collect::<Vec<_>>();

                let mut element_type = None;
                let mut all_same = true;
                for typ in &types {
                    if let Some(el) = element_type {
                        if el != typ {
                            all_same = false;
                            break;
                        }
                    } else {
                        element_type = Some(typ);
                    }
                }

                if all_same {
                    if let Some(typ) = element_type {
                        Type::List(Box::new(typ.clone()))
                    } else {
                        Type::Tuple(vec![])
                    }
                } else {
                    Type::Tuple(types)
                }
            }
            Value::Map(dict) => {
                let types = dict
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.canonical_type()))
                    .collect::<IndexMap<_, _>>();

                let mut element_type = None;
                let mut all_same = true;
                for (_, typ) in &types {
                    if let Some(el) = element_type {
                        if el != typ {
                            all_same = false;
                            break;
                        }
                    } else {
                        element_type = Some(typ);
                    }
                }

                if all_same {
                    if let Some(typ) = element_type {
                        Type::Dictionary(Box::new(typ.clone()))
                    } else {
                        Type::StrictRecord(IndexMap::new())
                    }
                } else {
                    Type::StrictRecord(types)
                }
            }
            Value::PatternMatches(_, _) => Type::Opaque("pattern match".to_string()),
            Value::NativePatternMatch(_) => Type::Opaque("native pattern match".to_string()),
            Value::Type(_) => Type::Opaque("type".to_string()),
        }
    }

    pub fn decode<T>(&self) -> Result<T, DecodeError>
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        let deserializer = crate::de::RyanDeserializer {
            value: std::borrow::Cow::Borrowed(self),
        };
        T::deserialize(deserializer)
    }
}

/// An iterator over a [`Value`], only in the cases that makes sense.
pub enum ValueIter<'a> {
    /// Iterator over a [`Value::List`] value.
    List(std::slice::Iter<'a, Value>),
    /// Iterator over a [`Value::Map`] value.
    Map(indexmap::map::Iter<'a, Rc<str>, Value>),
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::List(it) => it.next().cloned(),
            Self::Map(it) => it.next().map(|(key, value)| {
                Value::List(vec![Value::Text(key.clone()), value.clone()].into())
            }),
        }
    }
}

/// Error when the user tries to iterate over non-iterable values.
#[derive(Debug, Error)]
#[error("Value {val} is not iterable")]
pub struct NotIterable {
    val: Value,
}

pub struct TemplatedValue(pub Value);

impl Display for TemplatedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Value::Null => write!(f, "null")?,
            Value::Bool(b) => write!(f, "{b}")?,
            Value::Integer(int) => write!(f, "{int}")?,
            Value::Float(float) => write!(f, "{float}")?,
            Value::Text(text) => write!(f, "{text}")?,
            Value::List(list) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, list.iter())?;
                write!(f, "]")?;
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                crate::utils::fmt_map(f, map.iter())?;
                write!(f, "}}")?;
            }
            Value::PatternMatches(name, pattern_matches) => {
                write!(
                    f,
                    "![pattern {name} {}]",
                    pattern_matches
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(" | ")
                )?;
            }
            Value::NativePatternMatch(pattern_match) => {
                write!(f, "{pattern_match}")?;
            }
            Value::Type(r#type) => write!(f, "{type}")?,
        };

        Ok(())
    }
}
