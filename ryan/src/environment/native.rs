use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    rc::Rc,
};
use thiserror::Error;

use crate::{
    parser::{Pattern, Value},
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
        write!(f, "@{} {} => !?", self.identifier, self.pattern)
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

fn build_builtins() -> HashMap<Rc<str>, Value> {
    let mut builtins = HashMap::new();

    fn t(s: &str) -> Rc<str> {
        rc_world::str_to_rc(s)
    }

    let mut insert = |pat: NativePatternMatch| {
        builtins.insert(
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

    builtins
}

thread_local! {
    /// The Ryan default builtins that are supplied as "batteries included". All default
    /// builtins are guaranteed to finish executing and to not access the outside
    /// environment, in compliance to Ryan's key principles.
    pub static BUILTINS: Rc<HashMap<Rc<str>, Value>> = Rc::new(build_builtins());
}
