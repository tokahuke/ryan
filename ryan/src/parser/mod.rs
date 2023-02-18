mod binding;
mod block;
mod comprehension;
mod expression;
mod import;
mod literal;
mod operation;
mod pattern;
mod types;
mod value;

use indexmap::IndexMap;
use pest::{iterators::Pair, Parser as _};
use pest_derive::Parser;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::str;
use thiserror::Error;

use crate::environment::Environment;

pub use self::binding::Binding;
pub use self::block::Block;
pub use self::comprehension::ListComprehension;
pub use self::expression::{Dict, DictItem, Expression};
pub use self::import::{Format, Import};
pub use self::literal::Literal;
pub use self::operation::{
    BinaryOperation, BinaryOperator, PostfixOperation, PostfixOperator, PrefixOperation,
    PrefixOperator,
};
pub use self::pattern::{MatchDictItem, Pattern};
pub use self::types::{Type, TypeExpression};
pub use self::value::{NotIterable, NotRepresentable, PatternMatch, Value};

/// The Pest parser for Ryan.
#[allow(missing_docs)]
#[derive(Parser)]
#[grammar = "ryan.pest"] // relative to src
struct Parser;

/// An entry of a post-parsing error, logged by [`ErrorLogger`].
#[derive(Debug)]
pub struct ErrorEntry {
    /// The beginning and end of the offending code.
    pub span: (usize, usize),
    /// The error message for this error.
    pub error: String,
}

/// A logger of errors that happen post-parsing. Post parsing always succeeds, even with
/// a list of errors. It's the whole parsing processing that fails if there are
/// post-parsing errors.
#[derive(Debug, Default)]
pub struct ErrorLogger {
    /// The list of errors found during post-parsing, in the orders they were found.
    pub errors: Vec<ErrorEntry>,
}

impl ErrorLogger {
    fn absorb<T, E>(&mut self, pair: &Pair<Rule>, r: Result<T, E>) -> T
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
#[derive(Error)]
pub enum ParseError {
    /// A parse error found during the Pest parsing process. These are syntax errors.
    #[error("{0}")]
    During(pest::error::Error<Rule>),
    /// A parse errors found during the construction of the syntax tree. This covers some
    /// constraints not made explicit in the Pest grammar.
    #[error("{0:?}")]
    Post(ErrorLogger),
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::During(pest) => write!(f, "\n{pest}"),
            Self::Post(errors) => write!(f, "\n{errors:#?}"),
        }
    }
}

/// Parses a Ryan string and returns an abstract syntax tree (AST) object, represented by
/// its root, a [`Block`].
pub fn parse(s: &str) -> Result<Block, ParseError> {
    let mut parsed = Parser::parse(Rule::root, s).map_err(ParseError::During)?;
    let mut error_logger = ErrorLogger::default();
    let main = parsed.next().expect("there is always a matching token");
    let block = if !main.as_str().is_empty() {
        Block::parse(&mut error_logger, main.into_inner())
    } else {
        Block::null()
    };

    if error_logger.errors.is_empty() {
        Ok(block)
    } else {
        Err(ParseError::Post(error_logger))
    }
}

#[derive(Debug)]
enum Context {
    EvaluatingBinding(Rc<str>),
    SubstitutingPattern(Option<Rc<str>>),
    LoadingImport(Rc<str>),
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EvaluatingBinding(name) => write!(f, "Evaluating binding {name}"),
            Self::SubstitutingPattern(Some(name)) => write!(f, "Substituting pattern {name}"),
            Self::SubstitutingPattern(None) => write!(f, "Substituting anonymous pattern"),
            Self::LoadingImport(import) => write!(f, "Loading import {import:?}"),
        }
    }
}

#[derive(Debug)]
struct State<'a> {
    inherited: Option<&'a State<'a>>,
    bindings: IndexMap<Rc<str>, Value>,
    error: Option<String>,
    contexts: Vec<Context>,
    environment: Environment,
}

impl<'a> State<'a> {
    fn new(environment: Environment) -> State<'a> {
        State {
            inherited: None,
            bindings: IndexMap::new(),
            error: None,
            contexts: vec![],
            environment,
        }
    }

    fn absorb<T, E>(&mut self, r: Result<T, E>) -> Option<T>
    where
        E: ToString,
    {
        match r {
            Ok(t) => Some(t),
            Err(e) => {
                self.error = Some(e.to_string());
                None
            }
        }
    }

    fn raise<E>(&mut self, msg: E) -> Option<()>
    where
        E: ToString,
    {
        self.error = Some(msg.to_string());
        None
    }

    fn push_ctx(&mut self, ctx: Context) {
        self.contexts.push(ctx);
    }

    fn pop_ctx(&mut self) {
        self.contexts.pop();
    }

    fn try_get(&self, id: &str) -> Result<Value, String> {
        match self.bindings.get(id) {
            Some(bound) => Ok(bound.clone()),
            _ => {
                if let Some(inherited) = self.inherited.as_ref() {
                    inherited.try_get(id)
                } else if let Some(builtin) = self.environment.builtin(id) {
                    Ok(builtin)
                } else {
                    Err(format!("Variable `{id}` is undefined"))
                }
            }
        }
    }

    fn get(&mut self, id: &str) -> Option<Value> {
        self.absorb(self.try_get(id))
    }
}

/// An error that happens during the execution of a Ryan program.
#[derive(Debug, Error)]
pub struct EvalError {
    error: String,
    context: Vec<String>,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.error)?;

        if !self.context.is_empty() {
            writeln!(f)?;
            writeln!(f, "Context:")?;
            for line in &self.context {
                writeln!(f, "    - {line}")?;
            }
        }

        Ok(())
    }
}

/// Executes a block in a given environment, returning the resulting value.
pub fn eval(environment: Environment, block: &Block) -> Result<Value, EvalError> {
    let mut state = State::new(environment);

    if let Some(value) = block.eval(&mut state) {
        Ok(value)
    } else {
        Err(EvalError {
            error: state.error.expect("on backtracking, an error must be set"),
            context: state.contexts.iter().map(ToString::to_string).collect(),
        })
    }
}
