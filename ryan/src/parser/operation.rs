use pest::iterators::Pair;
use std::fmt::Display;
use std::rc::Rc;

use crate::rc_world;

use super::expression::Expression;
use super::value::Value;
use super::Context;
use super::ErrorLogger;
use super::Rule;
use super::State;

/// An operation involving two Ryan values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    /// Logical and.
    And,
    /// Logical or.
    Or,
    /// Strict equality.
    Equals,
    /// Strict inequality.
    NotEquals,
    // #[deprecated]
    /// Whether the value is of a certain type (to be deprecated).
    TypeMatches,
    /// Greater than comparison.
    GreaterThen,
    /// Greater than or equality comparison.
    GreaterEqual,
    /// Lesser than comparison.
    LesserThen,
    /// Lesser than or equality comparison.
    LesserEqual,
    /// Set inclusion
    IsContainedIn,
    /// Addition or concatenation.
    Plus,
    /// Subtraction.
    Minus,
    /// Multiplication.
    Times,
    /// Division.
    Divided,
    /// Remainder.
    Remainder,
    /// Returns the right side when the left side is `null`.
    Default,
    /// Pattern application.
    Juxtaposition,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "and")?,
            Self::Or => write!(f, "or")?,
            Self::Equals => write!(f, "==")?,
            Self::NotEquals => write!(f, "!=")?,
            Self::TypeMatches => write!(f, ":")?,
            Self::GreaterThen => write!(f, ">")?,
            Self::GreaterEqual => write!(f, ">=")?,
            Self::LesserThen => write!(f, "<")?,
            Self::LesserEqual => write!(f, "<=")?,
            Self::IsContainedIn => write!(f, "in")?,
            Self::Plus => write!(f, "+")?,
            Self::Minus => write!(f, "-")?,
            Self::Times => write!(f, "*")?,
            Self::Divided => write!(f, "/")?,
            Self::Remainder => write!(f, "%")?,
            Self::Default => write!(f, "?")?,
            Self::Juxtaposition => {}
        }

        Ok(())
    }
}

impl BinaryOperator {
    pub(super) fn parse(pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::andOp => BinaryOperator::And,
            Rule::orOp => BinaryOperator::Or,
            Rule::equalsOp => BinaryOperator::Equals,
            Rule::notEqualsOp => BinaryOperator::NotEquals,
            Rule::typeMatchesOp => BinaryOperator::TypeMatches,
            Rule::greaterOp => BinaryOperator::GreaterThen,
            Rule::greaterEqualOp => BinaryOperator::GreaterEqual,
            Rule::lesserOp => BinaryOperator::LesserThen,
            Rule::lesserEqualOp => BinaryOperator::LesserEqual,
            Rule::isContainedOp => BinaryOperator::IsContainedIn,
            Rule::plusOp => BinaryOperator::Plus,
            Rule::minusOp => BinaryOperator::Minus,
            Rule::timesOp => BinaryOperator::Times,
            Rule::dividedOp => BinaryOperator::Divided,
            Rule::remainderOp => BinaryOperator::Remainder,
            Rule::defaultOp => BinaryOperator::Default,
            Rule::juxtapositionOp => BinaryOperator::Juxtaposition,
            _ => unreachable!(),
        }
    }
}

/// An operation involving one Ryan value, where the value follows it.
#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOperator {
    /// Logical negation.
    Not,
}

impl Display for PrefixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Not => write!(f, "not")?,
        }

        Ok(())
    }
}

impl PrefixOperator {
    pub(super) fn parse(pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::notOp => PrefixOperator::Not,
            _ => unreachable!(),
        }
    }
}

/// An operation involving one Ryan value, where the value precedes it.
#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOperator {
    /// Get the value associated with a key in a dictionary using the familiar `.` notation.
    Access(Rc<str>),
    /// Access the value in a deeply nested Ryan object using the supplied path.
    Path(Vec<Expression>),
    /// Cast the value as integer.
    CastInt,
    /// Cast the value as float.
    CastFloat,
    /// Cast the value as text.
    CastText,
}

impl Display for PostfixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Access(field) => write!(f, ".{field}")?,
            Self::Path(exprs) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, exprs)?;
                write!(f, "]")?;
            }
            Self::CastInt => {
                write!(f, "as int")?;
            }
            Self::CastFloat => {
                write!(f, "as float")?;
            }
            Self::CastText => {
                write!(f, "as text")?;
            }
        }

        Ok(())
    }
}

impl PostfixOperator {
    pub(super) fn parse(logger: &mut ErrorLogger, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::accessOp => {
                let mut field = None;
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::identifier => field = Some(rc_world::str_to_rc(pair.as_str())),
                        _ => unreachable!(),
                    }
                }

                PostfixOperator::Access(
                    field.expect("there is always a field in an access operation"),
                )
            }
            Rule::pathOp => {
                let mut exprs = vec![];
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::expression => {
                            exprs.push(Expression::parse(logger, pair.into_inner()))
                        }
                        _ => unreachable!(),
                    }
                }

                PostfixOperator::Path(exprs)
            }
            Rule::castInt => PostfixOperator::CastInt,
            Rule::castFloat => PostfixOperator::CastFloat,
            Rule::castText => PostfixOperator::CastText,
            _ => unreachable!(),
        }
    }
}

/// An operation involving two Ryan expressions and a binary operator.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation {
    /// The left side of the operation.
    pub left: Expression,
    /// The binary operator to be used.
    pub op: BinaryOperator,
    /// The right side of the operation.
    pub right: Expression,
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let BinaryOperator::Juxtaposition = self.op {
            write!(f, "{} {}", self.left, self.right)
        } else {
            write!(f, "{} {} {}", self.left, self.op, self.right)
        }
    }
}

impl BinaryOperation {
    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        let left = self.left.eval(state)?;

        // These are short-circuiting operations...
        let left = match (left, self.op) {
            (Value::Bool(true), BinaryOperator::Or) => return Some(Value::Bool(true)),
            (Value::Bool(false), BinaryOperator::And) => return Some(Value::Bool(false)),
            (left, BinaryOperator::Default) if left != Value::Null => return Some(left),
            (left, _) => left, // not short-circuiting... carry on!
        };

        let right = self.right.eval(state)?;
        let result = match (left, self.op, right) {
            (Value::PatternMatches(id, pats), BinaryOperator::Juxtaposition, arg) => {
                state.push_ctx(Context::SubstitutingPattern(Some(id)));
                let mut evalued = None;
                let mut last_error = None;

                for pat in pats {
                    match pat.r#match(&arg, state)? {
                        Ok(found) => {
                            evalued = Some(found);
                            break;
                        }
                        Err(err) => last_error = Some(err),
                    }
                }

                if let Some(evalued) = evalued {
                    state.pop_ctx();
                    evalued
                } else {
                    state.raise(format!(
                        "{}",
                        last_error.expect("there is at least one patter in a pattern match")
                    ))?;
                    return None;
                }
            }
            (Value::NativePatternMatch(pat), BinaryOperator::Juxtaposition, arg) => {
                pat.r#match(arg, state)?
            }
            (value, BinaryOperator::Juxtaposition, Value::List(list)) => {
                match value.extract_path(&list) {
                    Ok(val) => val,
                    Err(err) => {
                        state.raise(err);
                        return None;
                    }
                }
            }
            (Value::Null, BinaryOperator::Default, val) => val.clone(),
            (first, BinaryOperator::Default, _) => first,
            (Value::Bool(left), BinaryOperator::Or, Value::Bool(right)) => {
                Value::Bool(left || right)
            }
            (Value::Bool(left), BinaryOperator::And, Value::Bool(right)) => {
                Value::Bool(left && right)
            }
            (left, BinaryOperator::Equals, right) => Value::Bool(left == right),
            (left, BinaryOperator::NotEquals, right) => Value::Bool(left != right),
            (left, BinaryOperator::TypeMatches, Value::Type(r#type)) => {
                Value::Bool(r#type.matches(&left))
            }
            (Value::Integer(left), BinaryOperator::GreaterThen, Value::Integer(right)) => {
                Value::Bool(left > right)
            }
            (Value::Integer(left), BinaryOperator::GreaterThen, Value::Float(right)) => {
                Value::Bool(left as f64 > right)
            }
            (Value::Float(left), BinaryOperator::GreaterThen, Value::Integer(right)) => {
                Value::Bool(left > right as f64)
            }
            (Value::Float(left), BinaryOperator::GreaterThen, Value::Float(right)) => {
                Value::Bool(left > right)
            }

            (Value::Integer(left), BinaryOperator::GreaterEqual, Value::Integer(right)) => {
                Value::Bool(left >= right)
            }
            (Value::Integer(left), BinaryOperator::GreaterEqual, Value::Float(right)) => {
                Value::Bool(left as f64 >= right)
            }
            (Value::Float(left), BinaryOperator::GreaterEqual, Value::Integer(right)) => {
                Value::Bool(left >= right as f64)
            }
            (Value::Float(left), BinaryOperator::GreaterEqual, Value::Float(right)) => {
                Value::Bool(left >= right)
            }

            (Value::Integer(left), BinaryOperator::LesserThen, Value::Integer(right)) => {
                Value::Bool(left < right)
            }
            (Value::Integer(left), BinaryOperator::LesserThen, Value::Float(right)) => {
                Value::Bool((left as f64) < right)
            }
            (Value::Float(left), BinaryOperator::LesserThen, Value::Integer(right)) => {
                Value::Bool(left < right as f64)
            }
            (Value::Float(left), BinaryOperator::LesserThen, Value::Float(right)) => {
                Value::Bool(left < right)
            }

            (Value::Integer(left), BinaryOperator::LesserEqual, Value::Integer(right)) => {
                Value::Bool(left <= right)
            }
            (Value::Integer(left), BinaryOperator::LesserEqual, Value::Float(right)) => {
                Value::Bool(left as f64 <= right)
            }
            (Value::Float(left), BinaryOperator::LesserEqual, Value::Integer(right)) => {
                Value::Bool(left <= right as f64)
            }
            (Value::Float(left), BinaryOperator::LesserEqual, Value::Float(right)) => {
                Value::Bool(left <= right)
            }

            (val, BinaryOperator::IsContainedIn, Value::List(list)) => {
                Value::Bool(list.iter().any(|item| *item == val))
            }
            (Value::Text(key), BinaryOperator::IsContainedIn, Value::Map(map)) => {
                Value::Bool(map.contains_key(&*key))
            }
            (Value::Text(sub), BinaryOperator::IsContainedIn, Value::Text(text)) => {
                Value::Bool(text.contains(&*sub))
            }

            (Value::Integer(left), BinaryOperator::Plus, Value::Integer(right)) => {
                Value::Integer(left + right)
            }
            (Value::Integer(left), BinaryOperator::Plus, Value::Float(right)) => {
                Value::Float(left as f64 + right)
            }
            (Value::Float(left), BinaryOperator::Plus, Value::Integer(right)) => {
                Value::Float(left + right as f64)
            }
            (Value::Float(left), BinaryOperator::Plus, Value::Float(right)) => {
                Value::Float(left + right)
            }

            (Value::Integer(left), BinaryOperator::Minus, Value::Integer(right)) => {
                Value::Integer(left - right)
            }
            (Value::Integer(left), BinaryOperator::Minus, Value::Float(right)) => {
                Value::Float(left as f64 - right)
            }
            (Value::Float(left), BinaryOperator::Minus, Value::Integer(right)) => {
                Value::Float(left - right as f64)
            }
            (Value::Float(left), BinaryOperator::Minus, Value::Float(right)) => {
                Value::Float(left - right)
            }

            (Value::Integer(left), BinaryOperator::Times, Value::Integer(right)) => {
                Value::Integer(left * right)
            }
            (Value::Integer(left), BinaryOperator::Times, Value::Float(right)) => {
                Value::Float(left as f64 * right)
            }
            (Value::Float(left), BinaryOperator::Times, Value::Integer(right)) => {
                Value::Float(left * right as f64)
            }
            (Value::Float(left), BinaryOperator::Times, Value::Float(right)) => {
                Value::Float(left * right)
            }

            (Value::Integer(_), BinaryOperator::Divided, Value::Integer(0)) => {
                Value::Float(f64::NAN)
            }
            (Value::Integer(left), BinaryOperator::Divided, Value::Integer(right)) => {
                Value::Integer(left / right)
            }
            (Value::Integer(left), BinaryOperator::Divided, Value::Float(right)) => {
                Value::Float(left as f64 / right)
            }
            (Value::Float(left), BinaryOperator::Divided, Value::Integer(right)) => {
                Value::Float(left / right as f64)
            }
            (Value::Float(left), BinaryOperator::Divided, Value::Float(right)) => {
                Value::Float(left / right)
            }

            (Value::Integer(_), BinaryOperator::Remainder, Value::Integer(0)) => {
                Value::Float(f64::NAN)
            }
            (Value::Integer(left), BinaryOperator::Remainder, Value::Integer(right)) => {
                Value::Integer(left % right)
            }
            (Value::Integer(left), BinaryOperator::Remainder, Value::Float(right)) => {
                Value::Float(left as f64 % right)
            }
            (Value::Float(left), BinaryOperator::Remainder, Value::Integer(right)) => {
                Value::Float(left % right as f64)
            }
            (Value::Float(left), BinaryOperator::Remainder, Value::Float(right)) => {
                Value::Float(left % right)
            }

            (Value::Text(left), BinaryOperator::Plus, Value::Text(right)) => {
                let cat = left.as_ref().to_string() + &right;
                Value::Text(rc_world::string_to_rc(cat))
            }
            (Value::List(left), BinaryOperator::Plus, Value::List(right)) => Value::List(Rc::from(
                left.iter()
                    .chain(right.as_ref())
                    .cloned()
                    .collect::<Vec<_>>(),
            )),
            (Value::Map(left), BinaryOperator::Plus, Value::Map(right)) => Value::Map(Rc::new(
                left.iter()
                    .chain(right.as_ref())
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )),
            (left, op, right) => {
                state.raise(format!(
                    "Operator `{}` cannot be applied to `{}` and `{}`",
                    op, left, right,
                ))?;
                return None;
            }
        };

        Some(result)
    }
}

/// An operation involving a Ryan expression and a prefix operator.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixOperation {
    /// The prefix operator.
    pub op: PrefixOperator,
    /// The expression on which the prefix operator is applied.
    pub right: Expression,
}

impl Display for PrefixOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.op, self.right)
    }
}

impl PrefixOperation {
    pub(super) fn eval(&self, state: &mut State) -> Option<Value> {
        let right = self.right.eval(state)?;

        let result = match (&self.op, &right) {
            (PrefixOperator::Not, Value::Bool(b)) => Value::Bool(!*b),
            _ => {
                state.raise(format!(
                    "Operator `{}` cannot be applied to `{}`",
                    self.op, right,
                ))?;
                return None;
            }
        };

        Some(result)
    }
}

/// An operation involving a Ryan expression and a postfix operator.
#[derive(Debug, Clone, PartialEq)]
pub struct PostfixOperation {
    /// The expression on which the postfix operator is applied.
    pub left: Expression,
    /// The postfix operator to be applied.
    pub op: PostfixOperator,
}

impl Display for PostfixOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.left, self.op)
    }
}

impl PostfixOperation {
    pub(super) fn eval(&self, state: &mut State) -> Option<Value> {
        let left = self.left.eval(state)?;

        let result = match (&left, &self.op) {
            (Value::Map(dict), PostfixOperator::Access(field)) => {
                if let Some(value) = dict.get(field) {
                    value.clone()
                } else {
                    state.raise(format!("Key `{}` not present in `{}`", field, left,))?;
                    return None;
                }
            }
            (left, PostfixOperator::Path(path)) => {
                match left.extract_path(
                    &path
                        .iter()
                        .map(|item| item.eval(state))
                        .collect::<Option<Vec<_>>>()?,
                ) {
                    Ok(value) => value,
                    Err(err) => {
                        state.raise(err);
                        return None;
                    }
                }
            }
            (Value::Bool(b), PostfixOperator::CastInt) => Value::Integer(*b as i64),
            (Value::Float(f), PostfixOperator::CastInt) => Value::Integer(*f as i64),
            (Value::Integer(i), PostfixOperator::CastInt) => Value::Integer(*i as i64),
            (Value::Bool(b), PostfixOperator::CastFloat) => Value::Float(*b as i64 as f64),
            (Value::Float(f), PostfixOperator::CastFloat) => Value::Float(*f as f64),
            (Value::Integer(i), PostfixOperator::CastFloat) => Value::Float(*i as f64),
            (left, PostfixOperator::CastText) => {
                Value::Text(rc_world::string_to_rc(left.to_string()))
            }
            _ => {
                state.raise(format!(
                    "Operator `{}` cannot be applied to `{}`",
                    self.op, left,
                ))?;
                return None;
            }
        };

        Some(result)
    }
}
