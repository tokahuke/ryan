use std::fmt::Display;
use std::rc::Rc;

use indexmap::IndexMap;
use pest::iterators::Pairs;

use crate::rc_world;
use crate::utils::QuotedStr;

use super::ErrorLogger;
use super::Rule;
use super::State;
use super::Value;

/// The type of a Ryan value.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Any type. Matches anything.
    Any,
    /// The `null` type. Matches only `null`.
    Null,
    /// A boolean.
    Bool,
    /// An integer.
    Integer,
    /// A float.
    Float,
    /// Some text.
    Text,
    /// A list where all elements are of the same type.
    List(Box<Type>),
    /// A dictionary where all the values are of the same type.
    Dictionary(Box<Type>),
    /// A list of given length where each element has a specific type.
    Tuple(Vec<Type>),
    /// A map where the given keys correspond to values of the given types.
    Record(IndexMap<String, Type>),
    /// A map containing only the given keys corresponding to values of the given types.
    StrictRecord(IndexMap<String, Type>),
    /// A value that can be of any of the values in a list.
    Or(Vec<Type>),
    /// A type which cannot be inspected. This variant cannot be created directly from Ryan code.
    Opaque(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "any")?,
            Self::Null => write!(f, "null")?,
            Self::Bool => write!(f, "bool")?,
            Self::Integer => write!(f, "int")?,
            Self::Float => write!(f, "float")?,
            Self::Text => write!(f, "text")?,
            Self::List(item) => write!(f, "[{item}]")?,
            Self::Dictionary(item) => write!(f, "{{{item}}}")?,
            Self::Tuple(items) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, items)?;
                write!(f, "]")?;
            }
            Self::Record(dict) => {
                write!(f, "{{ ")?;
                crate::utils::fmt_map(
                    f,
                    dict.iter().map(|(key, r#type)| (QuotedStr(&key), r#type)),
                )?;
                if dict.is_empty() {
                    write!(f, ".. }}")?;
                } else {
                    write!(f, ", .. }}")?;
                }
            }
            Self::StrictRecord(items) => {
                write!(f, "{{")?;
                crate::utils::fmt_map(
                    f,
                    items.iter().map(|(key, r#type)| (QuotedStr(&key), r#type)),
                )?;
                write!(f, "}}")?;
            }
            Self::Or(or_list) => {
                let first = or_list.first().expect("or type list cannot be empty");
                write!(f, "{first}")?;
                for item in or_list.iter().skip(1) {
                    write!(f, " | {item}")?;
                }
            }
            Self::Opaque(typ) => write!(f, "![type {typ}]")?,
        }

        Ok(())
    }
}

impl Type {
    /// Checks whether a given value corresponds to the given type.
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Any, _)
            | (Self::Null, Value::Null)
            | (Self::Bool, Value::Bool(_))
            | (Self::Integer, Value::Integer(_))
            | (Self::Float, Value::Float(_))
            | (Self::Text, Value::Text(_)) => true,
            (Self::List(r#type), Value::List(list)) => list.iter().all(|item| r#type.matches(item)),
            (Self::Dictionary(r#type), Value::Map(dict)) => {
                dict.iter().all(|(_, value)| r#type.matches(value))
            }
            (Self::Tuple(types), Value::List(list)) => {
                types.len() == list.len()
                    && types
                        .iter()
                        .zip(list.iter())
                        .all(|(r#type, item)| r#type.matches(item))
            }
            (Self::Record(record), Value::Map(dict)) => record.iter().all(|(key, r#type)| {
                dict.get(key.as_str())
                    .map(|value| r#type.matches(value))
                    .unwrap_or(false)
            }),
            (Self::StrictRecord(record), Value::Map(dict)) => record.iter().all(|(key, r#type)| {
                dict.get(key.as_str())
                    .map(|value| r#type.matches(value))
                    .unwrap_or(false)
            }),
            (Self::Or(or_list), value) => or_list.iter().any(|r#type| r#type.matches(value)),
            _ => false,
        }
    }
}

/// Ans expression returning a concrete Ryan type.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpression {
    /// Any type. Matches anything.
    Any,
    /// The `null` type. Matches only `null`.
    Null,
    /// A boolean.
    Bool,
    /// An integer.
    Integer,
    /// A float.
    Float,
    /// Some text.
    Text,
    /// A list where all elements are of the same type.
    List(Box<TypeExpression>),
    /// A dictionary where all the values are of the same type.
    Dictionary(Box<TypeExpression>),
    /// A list of given length where each element has a specific type.
    Tuple(Vec<TypeExpression>),
    /// A map where the given keys correspond to values of the given types.
    Record(IndexMap<String, TypeExpression>),
    /// A map containing only the given keys corresponding to values of the given types.
    StrictRecord(IndexMap<String, TypeExpression>),
    /// A value that can be of any of the values in a list.
    Or(Vec<TypeExpression>),
    /// A user-defined type stored in a given variable.
    Variable(Rc<str>),
}

impl Display for TypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "any")?,
            Self::Null => write!(f, "null")?,
            Self::Bool => write!(f, "bool")?,
            Self::Integer => write!(f, "int")?,
            Self::Float => write!(f, "float")?,
            Self::Text => write!(f, "text")?,
            Self::List(item) => write!(f, "[{item}]")?,
            Self::Dictionary(item) => write!(f, "{{{item}}}")?,
            Self::Tuple(items) => {
                write!(f, "(")?;
                crate::utils::fmt_list(f, items)?;
                write!(f, ")")?;
            }
            Self::Record(dict) => {
                write!(f, "{{ ")?;
                crate::utils::fmt_map(
                    f,
                    dict.iter().map(|(key, r#type)| (QuotedStr(&key), r#type)),
                )?;
                if dict.is_empty() {
                    write!(f, ".. }}")?;
                } else {
                    write!(f, ", .. }}")?;
                }
            }
            Self::StrictRecord(items) => {
                write!(f, "{{")?;
                crate::utils::fmt_map(
                    f,
                    items.iter().map(|(key, r#type)| (QuotedStr(&key), r#type)),
                )?;
                write!(f, "}}")?;
            }
            Self::Or(or_list) => {
                let first = or_list.first().expect("or type list cannot be empty");
                write!(f, "{first}")?;
                for item in or_list.iter().skip(1) {
                    write!(f, " | {item}")?;
                }
            }
            Self::Variable(id) => write!(f, "{id}")?,
        }

        Ok(())
    }
}

impl TypeExpression {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut or_list = Vec::with_capacity(1);

        for pair in pairs {
            let parsed_expression = match pair.as_rule() {
                Rule::optionalType => TypeExpression::Or(vec![
                    TypeExpression::parse(logger, pair.into_inner()),
                    TypeExpression::Null,
                ]),
                Rule::listType => {
                    TypeExpression::List(Box::new(TypeExpression::parse(logger, pair.into_inner())))
                }
                Rule::dictionaryType => TypeExpression::Dictionary(Box::new(
                    TypeExpression::parse(logger, pair.into_inner()),
                )),
                Rule::tupleType => TypeExpression::Tuple(
                    pair.into_inner()
                        .map(|pair| TypeExpression::parse(logger, pair.into_inner()))
                        .collect::<Vec<_>>(),
                ),
                Rule::recordType => TypeExpression::Record(
                    pair.into_inner()
                        .map(|pair| TypeItem::parse(logger, pair.into_inner()))
                        .map(|item| (item.identifier, item.r#type))
                        .collect(),
                ),
                Rule::strictRecordType => TypeExpression::StrictRecord(
                    pair.into_inner()
                        .map(|pair| TypeItem::parse(logger, pair.into_inner()))
                        .map(|item| (item.identifier, item.r#type))
                        .collect(),
                ),
                Rule::primitive => match pair.as_str() {
                    "null" => TypeExpression::Null,
                    "any" => TypeExpression::Any,
                    "bool" => TypeExpression::Bool,
                    "int" => TypeExpression::Integer,
                    "float" => TypeExpression::Float,
                    "number" => {
                        TypeExpression::Or(vec![TypeExpression::Integer, TypeExpression::Float])
                    }
                    "text" => TypeExpression::Text,
                    _ => unreachable!(),
                },
                Rule::identifier => TypeExpression::Variable(rc_world::str_to_rc(pair.as_str())),
                Rule::typeExpression => TypeExpression::parse(logger, pair.into_inner()),
                _ => unreachable!(),
            };

            match parsed_expression {
                TypeExpression::Or(other_list) => or_list.extend(other_list),
                something_else => or_list.push(something_else),
            }
        }

        if or_list.len() == 1 {
            or_list.pop().unwrap()
        } else {
            Self::Or(or_list)
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &[Rc<str>],
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        if let Self::Variable(id) = self {
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

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Type> {
        let evalued = match self {
            Self::Any => Type::Any,
            Self::Null => Type::Null,
            Self::Bool => Type::Bool,
            Self::Integer => Type::Integer,
            Self::Float => Type::Float,
            Self::Text => Type::Text,
            Self::List(item) => Type::List(Box::new(item.eval(state)?)),
            Self::Dictionary(item) => Type::Dictionary(Box::new(item.eval(state)?)),
            Self::Tuple(tuple) => Type::Tuple(
                tuple
                    .iter()
                    .map(|item| item.eval(state))
                    .collect::<Option<Vec<_>>>()?,
            ),
            Self::Record(record) => Type::Record(
                record
                    .iter()
                    .map(|(id, expr)| expr.eval(state).map(|r#type| (id.clone(), r#type)))
                    .collect::<Option<IndexMap<_, _>>>()?,
            ),
            Self::StrictRecord(record) => Type::StrictRecord(
                record
                    .iter()
                    .map(|(id, expr)| expr.eval(state).map(|r#type| (id.clone(), r#type)))
                    .collect::<Option<IndexMap<_, _>>>()?,
            ),
            Self::Or(items) => Type::Or(
                items
                    .iter()
                    .map(|item| item.eval(state))
                    .collect::<Option<Vec<_>>>()?,
            ),
            Self::Variable(identifier) => match state.get(&identifier)? {
                Value::Type(r#type) => r#type,
                val => {
                    state.raise(format!("The value `{val}` is not a type"))?;
                    return None;
                }
            },
        };

        Some(evalued)
    }
}

struct TypeItem {
    identifier: String,
    r#type: TypeExpression,
}

impl TypeItem {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut identifier = None;
        let mut r#type = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::identifier => identifier = Some(pair.as_str().to_owned()),
                Rule::text => {
                    identifier = Some(logger.absorb(&pair, crate::utils::unescape(pair.as_str())))
                }
                Rule::typeExpression => {
                    r#type = Some(TypeExpression::parse(logger, pair.into_inner()))
                }
                _ => unreachable!(),
            }
        }

        TypeItem {
            identifier: identifier.expect("there is always an identifier in a type item"),
            r#type: r#type.expect("there is always a type in a type item"),
        }
    }
}
