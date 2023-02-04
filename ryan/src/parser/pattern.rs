use pest::iterators::Pairs;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use thiserror::Error;

use crate::rc_world;
use crate::utils::QuotedStr;

use super::literal::Literal;
use super::types::Type;
use super::types::TypeExpression;
use super::value::Value;
use super::ErrorLogger;
use super::Rule;
use super::State;

#[derive(Debug, Error)]
pub enum BindError {
    #[error("Variable {id} bound to {val} is not of type {typ}")]
    WrongType { id: Rc<str>, val: Value, typ: Type },
    #[error("Pattern expected list with {expected} elements, got list with {got}")]
    WrongListLength { expected: usize, got: usize },
    #[error("Pattern expected list with at least {expected} elements, got list with {got}")]
    TooFewValuesInList { expected: usize, got: usize },
    #[error("Pattern expect key {key} in {value}")]
    MissingKey { key: Rc<str>, value: Value },
    #[error("Pattern expected a strict match of {pattern} on {value}")]
    MatchIsNonStrict { pattern: Pattern, value: Value },
    #[error("Pattern expected {pattern}, got {value}")]
    NoMatch { pattern: Pattern, value: Value },
}

/// An expression expecting a certain structure of a given value and optionally binding
/// variables to selected bits and pieces of this value.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Matches any value and provides no biding. This is represented by `_` in Ryan.
    Wildcard,
    /// Matches any value optionally conforming to a given type expression and binds it
    /// to a variable of a given name. This is represented by, e.g, `x` or `x: int` in
    /// Ryan.
    Identifier(Rc<str>, Option<TypeExpression>),
    /// Expects a literal value. This is represented by, e.g, `1` or `"abc"` in Ryan.
    Literal(Literal),
    /// Expects a list of fixed size and proceeds to bind each of its elements to
    /// patterns. This is represented by, e.g., `[a, b, c]` in Ryan.
    MatchList(Vec<Pattern>),
    /// Expects a list of at least a given size and proceeds to bind the beginning of the
    /// list to patterns. This is represented by, e.g., `[a, b, c, ..]` in Ryan.
    MatchHead(Vec<Pattern>),
    /// Expects a list of at least a given size and proceeds to bind the end of the list
    /// to patterns. This is represented by, e.g., `[.., a, b, c]` in Ryan.
    MatchTail(Vec<Pattern>),
    /// Expects a dictionary with at least the provided keys and proceeds to bind each
    /// value to a pattern. This is represented by, e.g., `{ a, "b": c, .. }` in Ryan.
    MatchDict(Vec<MatchDictItem>),
    /// Expects a dictionary with exactly the provided keys and proceeds to bind each
    /// value to a pattern. This is represented by, e.g., `{ a, "b": c }` in Ryan.
    MatchDictStrict(Vec<MatchDictItem>),
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wildcard => write!(f, "_")?,
            Self::Identifier(id, None) => write!(f, "{id}")?,
            Self::Identifier(id, Some(t)) => write!(f, "{id}: {t}")?,
            Self::Literal(lit) => write!(f, "{lit}")?,
            Self::MatchList(list) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, list)?;
                write!(f, "]")?;
            }
            Self::MatchHead(list) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, list)?;
                if list.is_empty() {
                    write!(f, " .. ]")?;
                } else {
                    write!(f, ", .. ]")?;
                }
            }
            Self::MatchTail(list) => {
                if list.is_empty() {
                    write!(f, "[ ..")?;
                } else {
                    write!(f, "[ .., ")?;
                }
                crate::utils::fmt_list(f, list)?;
                write!(f, "]")?;
            }
            Self::MatchDict(dict) => {
                write!(f, "{{ ")?;
                crate::utils::fmt_map(
                    f,
                    dict.iter()
                        .map(|item| (QuotedStr(&item.key), &item.pattern)),
                )?;
                if dict.is_empty() {
                    write!(f, ".. }}")?;
                } else {
                    write!(f, ", .. }}")?;
                }
            }
            Self::MatchDictStrict(dict) => {
                write!(f, "{{")?;
                crate::utils::fmt_map(
                    f,
                    dict.iter()
                        .map(|item| (QuotedStr(&item.key), &item.pattern)),
                )?;
                write!(f, "}}")?;
            }
        }

        Ok(())
    }
}

impl Pattern {
    pub(super) fn parse(error_logger: &mut ErrorLogger, mut pairs: Pairs<'_, Rule>) -> Self {
        let pair = pairs.next().expect("there is always a token in a pattern");

        match pair.as_rule() {
            Rule::wildcard => Pattern::Wildcard,
            Rule::matchIdentifier => {
                let mut identifier = None;
                let mut type_guard = None;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::identifier => identifier = Some(rc_world::str_to_rc(pair.as_str())),
                        Rule::typeExpression => {
                            type_guard =
                                Some(TypeExpression::parse(error_logger, pair.into_inner()))
                        }
                        _ => unreachable!(),
                    }
                }

                Pattern::Identifier(
                    identifier.expect("identifier match has an identifier"),
                    type_guard,
                )
            }
            Rule::literal => Pattern::Literal(Literal::parse(error_logger, pair.into_inner())),
            Rule::matchList => Pattern::MatchList(
                pair.into_inner()
                    .map(|pair| Pattern::parse(error_logger, pair.into_inner()))
                    .collect(),
            ),
            Rule::matchHead => Pattern::MatchHead(
                pair.into_inner()
                    .map(|pair| Pattern::parse(error_logger, pair.into_inner()))
                    .collect(),
            ),
            Rule::matchTail => Pattern::MatchTail(
                pair.into_inner()
                    .map(|pair| Pattern::parse(error_logger, pair.into_inner()))
                    .collect(),
            ),
            Rule::matchDict => Pattern::MatchDict(
                pair.into_inner()
                    .map(|pair| MatchDictItem::parse(error_logger, pair.into_inner()))
                    .collect(),
            ),
            Rule::matchDictStrict => Pattern::MatchDictStrict(
                pair.into_inner()
                    .map(|pair| MatchDictItem::parse(error_logger, pair.into_inner()))
                    .collect(),
            ),
            _ => unreachable!(),
        }
    }

    pub(super) fn provided(&self, identifiers: &mut Vec<Rc<str>>) {
        match self {
            Self::Wildcard => {}
            Self::Identifier(id, _) => identifiers.push(id.clone()),
            Self::Literal(_) => {}
            Self::MatchList(list) => {
                for item in list {
                    item.provided(identifiers);
                }
            }
            Self::MatchHead(list) => {
                for item in list {
                    item.provided(identifiers);
                }
            }
            Self::MatchTail(list) => {
                for item in list {
                    item.provided(identifiers);
                }
            }
            Self::MatchDict(dict) => {
                for item in dict {
                    item.pattern.provided(identifiers);
                }
            }
            Self::MatchDictStrict(dict) => {
                for item in dict {
                    item.pattern.provided(identifiers);
                }
            }
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &[Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        if let Self::Identifier(_, Some(type_guard)) = self {
            type_guard.capture(state, provided, values)?;
        }

        Some(())
    }

    pub(super) fn bind(
        &self,
        value: &Value,
        bindings: &mut HashMap<Rc<str>, Value>,
        state: &mut State<'_>,
    ) -> Option<Result<(), BindError>> {
        match (self, value) {
            (Pattern::Wildcard, _) => {}
            (Pattern::Identifier(id, type_guard), val) => {
                if let Some(guard) = type_guard {
                    let typ = guard.eval(state)?;
                    if !typ.matches(&val) {
                        return Some(Err(BindError::WrongType {
                            id: id.clone(),
                            val: val.clone(),
                            typ,
                        }));
                    }
                }

                bindings.insert(id.clone(), val.clone());
            }
            (Pattern::Literal(lit), val) if val.matches(lit) => {}
            (Pattern::MatchList(pat_list), Value::List(val_list)) => {
                if pat_list.len() == val_list.len() {
                    for (pat, val) in pat_list.iter().zip(val_list.iter()) {
                        if let Err(err) = pat.bind(val, bindings, state)? {
                            return Some(Err(err));
                        }
                    }
                } else {
                    return Some(Err(BindError::WrongListLength {
                        expected: pat_list.len(),
                        got: val_list.len(),
                    }));
                }
            }
            (Pattern::MatchHead(pat_list), Value::List(val_list)) => {
                if pat_list.len() <= val_list.len() {
                    for (pat, val) in pat_list.iter().zip(val_list.iter()) {
                        if let Err(err) = pat.bind(val, bindings, state)? {
                            return Some(Err(err));
                        }
                    }
                } else {
                    return Some(Err(BindError::TooFewValuesInList {
                        expected: pat_list.len(),
                        got: val_list.len(),
                    }));
                }
            }
            (Pattern::MatchTail(pat_list), Value::List(val_list)) => {
                if pat_list.len() <= val_list.len() {
                    for (pat, val) in pat_list.iter().rev().zip(val_list.iter().rev()) {
                        if let Err(err) = pat.bind(val, bindings, state)? {
                            return Some(Err(err));
                        }
                    }
                } else {
                    return Some(Err(BindError::TooFewValuesInList {
                        expected: pat_list.len(),
                        got: val_list.len(),
                    }));
                }
            }
            (Pattern::MatchDict(list), Value::Map(val_dict)) => {
                for item in list {
                    if let Some(val) = val_dict.get(&item.key) {
                        if let Err(err) = item.pattern.bind(val, bindings, state)? {
                            return Some(Err(err));
                        }
                    } else {
                        return Some(Err(BindError::MissingKey {
                            key: item.key.clone(),
                            value: Value::Map(val_dict.clone()),
                        }));
                    }
                }
            }
            (Pattern::MatchDictStrict(list), Value::Map(val_dict)) => {
                for item in list {
                    if let Some(val) = val_dict.get(&item.key) {
                        if let Err(err) = item.pattern.bind(val, bindings, state)? {
                            return Some(Err(err));
                        }
                    } else {
                        return Some(Err(BindError::MissingKey {
                            key: item.key.clone(),
                            value: Value::Map(val_dict.clone()),
                        }));
                    }
                }

                if list.len() != val_dict.len() {
                    return Some(Err(BindError::MatchIsNonStrict {
                        pattern: self.clone(),
                        value: value.clone(),
                    }));
                }
            }
            (_, _) => {
                return Some(Err(BindError::NoMatch {
                    pattern: self.clone(),
                    value: value.clone(),
                }))
            }
        }

        Some(Ok(()))
    }
}

/// A pattern matching a dictionary entry. This can take the form of `x`, which binds the
/// value associated to the key `x` to the variable `x` or `x: pattern` which bind the
/// value associated with `x`to another pattern. Of note is that, in this position,
/// `pattern` cannot be an identifier pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchDictItem {
    /// The key which must exist in the dictionary.
    pub key: Rc<str>,
    /// The pattern to which the value associated with the key will be matched against.
    pub pattern: Pattern,
}

impl MatchDictItem {
    fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut key = None;
        let mut text = None;
        let mut pattern = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::identifier => key = Some(rc_world::str_to_rc(pair.as_str())),
                Rule::pattern => pattern = Some(Pattern::parse(logger, pair.into_inner())),
                Rule::text => {
                    text = Some(rc_world::string_to_rc(
                        logger.absorb(&pair, snailquote::unescape(pair.as_str())),
                    ))
                }
                Rule::matchIdentifier => {
                    // TODO: code repeated from Pattern::parse
                    let mut identifier = None;
                    let mut type_guard = None;

                    for pair in pair.into_inner() {
                        match pair.as_rule() {
                            Rule::identifier => {
                                identifier = Some(rc_world::str_to_rc(pair.as_str()))
                            }
                            Rule::typeExpression => {
                                type_guard = Some(TypeExpression::parse(logger, pair.into_inner()))
                            }
                            _ => unreachable!(),
                        }
                    }

                    let identifier = identifier.expect("identifier match has an identifier");

                    return MatchDictItem {
                        key: identifier.clone(),
                        pattern: Pattern::Identifier(identifier, type_guard),
                    };
                }
                _ => unreachable!(),
            }
        }

        MatchDictItem {
            key: key
                .as_ref()
                .map(Rc::clone)
                .or(text)
                .expect("a match dict item always has a key"),
            pattern: pattern
                // .or(key.map(Pattern::Identifier))
                .expect("a match dict always has a pattern"),
        }
    }
}
