use std::{
    cmp,
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    rc::Rc,
};
use thiserror::Error;

use crate::{
    parser::{NotIterable, Pattern, TypeExpression, Value},
    rc_world,
};

/// A native pattern match. It matches a Ryan value to a given pattern and, if there is
/// a match, applies a supplied closure to the value. Use this type to create your own
/// extensions and built-in functions to Ryan.
pub struct NativePatternMatch {
    /// The name by which users will call this pattern match in their code.
    pub identifier: Rc<str>,
    /// The pattern to which input values must comply to.
    pub pattern: Pattern,
    /// The native function mapping the input value to the output value.
    pub func: Box<dyn Fn(Value) -> Result<Value, Box<dyn Error + 'static>>>,
}

impl Display for NativePatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "![native pattern {} {}]", self.identifier, self.pattern)
    }
}

impl Debug for NativePatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl PartialEq for NativePatternMatch {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier && self.pattern == other.pattern
    }
}

impl NativePatternMatch {
    /// Creates a new native pattern match given a name, a pattern and a mapping function.
    pub fn new<F, E>(name: &str, pattern: Pattern, f: F) -> NativePatternMatch
    where
        F: 'static + Fn(Value) -> Result<Value, E>,
        E: 'static + Error,
    {
        NativePatternMatch {
            identifier: rc_world::str_to_rc(name),
            pattern,
            func: Box::new(move |v| f(v).map_err(|e| Box::new(e).into())),
        }
    }
}

/// A wrapper around a string that implements [`Error`]. Use this type to conveniently
/// throw log-and-forget errors from your extensions.
#[derive(Debug, Error)]
pub struct BuiltinErrorMsg(String);

impl Display for BuiltinErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn build_built_ins() -> HashMap<Rc<str>, Value> {
    let mut built_ins = HashMap::new();

    fn t(s: &str) -> Rc<str> {
        rc_world::str_to_rc(s)
    }

    let mut insert = |pat: NativePatternMatch| {
        built_ins.insert(
            pat.identifier.clone(),
            Value::NativePatternMatch(pat.into()),
        )
    };

    insert(NativePatternMatch::new(
        "fmt",
        Pattern::Identifier(t("x"), None),
        move |value| {
            Ok(Value::Text(rc_world::string_to_rc(value.to_string()))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "len",
        Pattern::Identifier(t("x"), None),
        move |value| {
            let len = match value {
                Value::List(list) => list.len() as i64,
                Value::Map(map) => map.len() as i64,
                Value::Text(text) => text.len() as i64,
                _ => return Err(BuiltinErrorMsg(format!("Value `{value}` has no length"))),
            };

            Ok(Value::Integer(len))
        },
    ));
    insert(NativePatternMatch::new(
        "range",
        Pattern::MatchList(vec![
            Pattern::Identifier(t("start"), None),
            Pattern::Identifier(t("end"), None),
        ]),
        move |value| match value {
            Value::List(range) => match &*range {
                [Value::Integer(start), Value::Integer(end)] => {
                    Ok(Value::List((*start..*end).map(Value::Integer).collect()))
                }
                bad => Err(BuiltinErrorMsg(format!("List `{bad:?}` cannot be a range"))),
            },
            _ => Err(BuiltinErrorMsg(format!(
                "Value `{value}` cannot be a range"
            ))),
        },
    ));
    insert(NativePatternMatch::new(
        "zip",
        Pattern::MatchList(vec![
            Pattern::Identifier(t("left"), None),
            Pattern::Identifier(t("right"), None),
        ]),
        move |value| {
            let Value::List(list) = value else {
                unreachable!()
            };
            let [left, right] = &*list else {
                unreachable!()
            };

            let zipped: Value = left
                .iter()?
                .zip(right.iter()?)
                .map(|(left, right)| Value::List(vec![left, right].into()))
                .collect();

            Ok(zipped) as Result<_, NotIterable>
        },
    ));
    insert(NativePatternMatch::new(
        "enumerate",
        Pattern::Identifier(t("x"), None),
        move |value| {
            let enumerated: Value = value
                .iter()?
                .enumerate()
                .map(|(i, val)| Value::List(vec![Value::Integer(i as i64), val].into()))
                .collect();
            Ok(enumerated) as Result<_, NotIterable>
        },
    ));
    insert(NativePatternMatch::new(
        "sum",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::List(Box::new(TypeExpression::Or(vec![
                TypeExpression::Float,
                TypeExpression::Integer,
            ])))),
        ),
        move |value| {
            let mut sum = Value::Integer(0);

            for val in value.iter()? {
                sum = match (val, sum) {
                    (Value::Integer(val), Value::Integer(sum)) => Value::Integer(val + sum),
                    (Value::Float(val), Value::Integer(sum)) => Value::Float(val + sum as f64),
                    (Value::Integer(val), Value::Float(sum)) => Value::Float(val as f64 + sum),
                    (Value::Float(val), Value::Float(sum)) => Value::Float(val + sum),
                    _ => unreachable!(),
                }
            }

            Ok(sum) as Result<_, NotIterable>
        },
    ));
    insert(NativePatternMatch::new(
        "max",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::List(Box::new(TypeExpression::Or(vec![
                TypeExpression::Float,
                TypeExpression::Integer,
            ])))),
        ),
        move |value| {
            let mut max = Value::Integer(0);

            for val in value.iter()? {
                max = match (val, max) {
                    (Value::Integer(val), Value::Integer(max)) => {
                        Value::Integer(i64::max(val, max))
                    }
                    (Value::Float(val), Value::Integer(max)) => {
                        Value::Float(f64::max(val, max as f64))
                    }
                    (Value::Integer(val), Value::Float(max)) => {
                        Value::Float(f64::max(val as f64, max))
                    }
                    (Value::Float(val), Value::Float(max)) => Value::Float(f64::max(val, max)),
                    _ => unreachable!(),
                }
            }

            Ok(max) as Result<_, NotIterable>
        },
    ));
    insert(NativePatternMatch::new(
        "min",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::List(Box::new(TypeExpression::Or(vec![
                TypeExpression::Float,
                TypeExpression::Integer,
            ])))),
        ),
        move |value| {
            let mut min = Value::Integer(0);

            for val in value.iter()? {
                min = match (val, min) {
                    (Value::Integer(val), Value::Integer(min)) => {
                        Value::Integer(i64::min(val, min))
                    }
                    (Value::Float(val), Value::Integer(min)) => {
                        Value::Float(f64::min(val, min as f64))
                    }
                    (Value::Integer(val), Value::Float(min)) => {
                        Value::Float(f64::min(val as f64, min))
                    }
                    (Value::Float(val), Value::Float(min)) => Value::Float(f64::min(val, min)),
                    _ => unreachable!(),
                }
            }

            Ok(min) as Result<_, NotIterable>
        },
    ));
    insert(NativePatternMatch::new(
        "all",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::List(Box::new(TypeExpression::Or(vec![
                TypeExpression::Bool,
            ])))),
        ),
        move |value| {
            for val in value.iter()? {
                if let Value::Bool(false) = val {
                    return Ok(Value::Bool(false));
                }
            }

            Ok(Value::Bool(true)) as Result<_, NotIterable>
        },
    ));
    insert(NativePatternMatch::new(
        "any",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::List(Box::new(TypeExpression::Or(vec![
                TypeExpression::Bool,
            ])))),
        ),
        move |value| {
            for val in value.iter()? {
                if let Value::Bool(true) = val {
                    return Ok(Value::Bool(true));
                }
            }

            Ok(Value::Bool(false)) as Result<_, NotIterable>
        },
    ));

    #[derive(Debug, Error)]
    #[error("Value {a} cannot be compared with {b}")]
    struct NotComparable {
        a: Value,
        b: Value,
    }

    insert(NativePatternMatch::new(
        "sort",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::List(Box::new(TypeExpression::Any))),
        ),
        move |value| {
            let Value::List(list) = value else {
                unreachable!()
            };
            let mut list = list.to_vec();
            let mut bad_comp = None;
            list.sort_by(|a, b| {
                if let Some(cmp) = a.partial_cmp(b) {
                    cmp
                } else {
                    bad_comp = Some(NotComparable {
                        a: a.clone(),
                        b: b.clone(),
                    });
                    cmp::Ordering::Greater
                }
            });

            if let Some(error) = bad_comp {
                Err(error)
            } else {
                Ok(Value::List(list.into()))
            }
        },
    ));
    insert(NativePatternMatch::new(
        "keys",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::Dictionary(Box::new(TypeExpression::Any))),
        ),
        move |value| {
            let Value::Map(dict) = value else {
                unreachable!()
            };
            let keys: Vec<_> = dict
                .keys()
                .map(|key| Value::Text(rc_world::str_to_rc(key)))
                .collect();

            Ok(Value::List(keys.into())) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "values",
        Pattern::Identifier(
            t("x"),
            Some(TypeExpression::Dictionary(Box::new(TypeExpression::Any))),
        ),
        move |value| {
            let Value::Map(dict) = value else {
                unreachable!()
            };
            let keys: Vec<_> = dict.values().cloned().collect();

            Ok(Value::List(keys.into())) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "join",
        Pattern::Identifier(t("sep"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(separator) = value else {
                unreachable!()
            };

            Ok(Value::NativePatternMatch(Rc::new(NativePatternMatch::new(
                "join$ret",
                Pattern::Identifier(
                    t("x"),
                    Some(TypeExpression::List(Box::new(TypeExpression::Text))),
                ),
                move |value| {
                    let mut iter = value.iter()?;
                    let mut string = String::new();

                    if let Some(val) = iter.next() {
                        let Value::Text(text) = val else {
                            unreachable!()
                        };
                        string += text.as_ref();
                    }

                    for val in iter {
                        let Value::Text(text) = val else {
                            unreachable!()
                        };
                        string += &*separator;
                        string += &*text;
                    }

                    Ok(Value::Text(rc_world::string_to_rc(string))) as Result<_, NotIterable>
                },
            )))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "split",
        Pattern::Identifier(t("sep"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(separator) = value else {
                unreachable!()
            };

            Ok(Value::NativePatternMatch(Rc::new(NativePatternMatch::new(
                "split$ret",
                Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
                move |value| {
                    let Value::Text(text) = value else {
                        unreachable!()
                    };

                    let split: Vec<_> = text
                        .split(&*separator)
                        .map(|part| Value::Text(rc_world::str_to_rc(part)))
                        .collect();
                    Ok(Value::List(split.into())) as Result<_, NotIterable>
                },
            )))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "trim",
        Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(text) = value else {
                unreachable!()
            };

            Ok(Value::Text(rc_world::str_to_rc(
                text.trim_start().trim_end(),
            ))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "trim_start",
        Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(text) = value else {
                unreachable!()
            };

            Ok(Value::Text(rc_world::str_to_rc(text.trim_start()))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "trim_end",
        Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(text) = value else {
                unreachable!()
            };

            Ok(Value::Text(rc_world::str_to_rc(text.trim_end()))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "starts_with",
        Pattern::Identifier(t("prefix"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(prefix) = value else {
                unreachable!()
            };

            Ok(Value::NativePatternMatch(Rc::new(NativePatternMatch::new(
                "starts_with$ret",
                Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
                move |value| {
                    let Value::Text(text) = value else {
                        unreachable!()
                    };

                    let starts_with = text.starts_with(&*prefix);
                    Ok(Value::Bool(starts_with)) as Result<_, NotIterable>
                },
            )))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "ends_with",
        Pattern::Identifier(t("postfix"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(postfix) = value else {
                unreachable!()
            };

            Ok(Value::NativePatternMatch(Rc::new(NativePatternMatch::new(
                "ends_with$ret",
                Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
                move |value| {
                    let Value::Text(text) = value else {
                        unreachable!()
                    };

                    let starts_with = text.ends_with(&*postfix);
                    Ok(Value::Bool(starts_with)) as Result<_, NotIterable>
                },
            )))) as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "lowercase",
        Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(text) = value else {
                unreachable!()
            };

            Ok(Value::Text(rc_world::string_to_rc(text.to_lowercase())))
                as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "uppercase",
        Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
        move |value| {
            let Value::Text(text) = value else {
                unreachable!()
            };

            Ok(Value::Text(rc_world::string_to_rc(text.to_uppercase())))
                as Result<_, BuiltinErrorMsg>
        },
    ));
    insert(NativePatternMatch::new(
        "replace",
        Pattern::MatchList(vec![
            Pattern::Identifier(t("find"), Some(TypeExpression::Text)),
            Pattern::Identifier(t("subst"), Some(TypeExpression::Text)),
        ]),
        move |value| {
            let Value::List(list) = value else {
                unreachable!()
            };
            let [Value::Text(find), Value::Text(subst)] = &*list else {
                unreachable!()
            };
            let find = find.clone();
            let subst = subst.clone();

            Ok(Value::NativePatternMatch(Rc::new(NativePatternMatch::new(
                "replace$ret",
                Pattern::Identifier(t("x"), Some(TypeExpression::Text)),
                move |value| {
                    let Value::Text(text) = value else {
                        unreachable!()
                    };

                    let replaced = text.replace(find.as_ref(), &subst);
                    Ok(Value::Text(rc_world::string_to_rc(replaced))) as Result<_, NotIterable>
                },
            )))) as Result<_, BuiltinErrorMsg>
        },
    ));

    built_ins
}

thread_local! {
    /// The Ryan default built_ins that are supplied as "batteries included". All default
    /// built_ins are guaranteed to finish executing and to not access the outside
    /// environment, in compliance to Ryan's key principles.
    pub static BUILT_INS: Rc<HashMap<Rc<str>, Value>> = Rc::new(build_built_ins());
}
