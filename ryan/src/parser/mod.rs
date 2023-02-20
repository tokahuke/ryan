mod binding;
mod block;
mod comprehension;
mod error;
mod expression;
mod import;
mod literal;
mod operation;
mod pattern;
mod types;
mod value;

use indexmap::IndexMap;
use pest::Parser as _;
use pest_derive::Parser;
use std::fmt::Display;
use std::rc::Rc;
use std::str;
use thiserror::Error;

use crate::environment::Environment;
use crate::rc_world;

pub use self::binding::Binding;
pub use self::block::Block;
pub use self::comprehension::ListComprehension;
pub use self::error::{ErrorEntry, ErrorLogger, ParseError};
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

impl Rule {
    /// A human-readable name for each grammar rule.
    fn name(&self) -> &'static str {
        match self {
            Rule::EOI => "end of input",
            Rule::WHITESPACE => "whitespace",
            Rule::COMMENT => "a comment",
            Rule::root => "a Ryan program",
            Rule::main => "a Ryan program",
            Rule::literal => "a literal value",
            Rule::unsigned => "an unsigned number",
            Rule::null => "null",
            Rule::sign => "`+` or `-`",
            Rule::number => "a number",
            Rule::bool => "a boolean",
            Rule::escaped => "the interior of escaped text",
            Rule::controlCode => "a control code in escaped text",
            Rule::text => "text",
            Rule::identifier => "a variable name",
            Rule::identifierStr => "a variable name",
            Rule::reserved => "a reserved keyword",
            Rule::expression => "an expression",
            Rule::binaryOp => "a binary operation",
            Rule::orOp => "`or`",
            Rule::andOp => "`and`",
            Rule::equalsOp => "`==`",
            Rule::notEqualsOp => "`!=`",
            Rule::typeMatchesOp => "`:`",
            Rule::greaterOp => "`>`",
            Rule::greaterEqualOp => "`>=`",
            Rule::lesserOp => "`<`",
            Rule::lesserEqualOp => "`<=`",
            Rule::isContainedOp => "`in`",
            Rule::plusOp => "`+`",
            Rule::minusOp => "`-`",
            Rule::timesOp => "`*`",
            Rule::dividedOp => "`/`",
            Rule::remainderOp => "`%`",
            Rule::defaultOp => "`?`",
            Rule::juxtapositionOp => "a juxtaposition",
            Rule::prefixOp => "a prefix operator",
            Rule::notOp => "`not`",
            Rule::postfixOp => "a postfix operator",
            Rule::accessOp => "list or map access",
            Rule::pathOp => "list or map access",
            Rule::term => "an expression term",
            Rule::list => "a list",
            Rule::dictItem => "a dictionary item",
            Rule::dict => "a dictionary",
            Rule::conditional => "`if ... then ... else ...`",
            Rule::listComprehension => "a list comprehension",
            Rule::dictComprehension => "a dictionary comprehension",
            Rule::forClause => "a `for` clause",
            Rule::ifGuard => "an `if` guard",
            Rule::keyValueClause => "a key-value clause",
            Rule::pattern => "a pattern match",
            Rule::wildcard => "a wildcard pattern patch",
            Rule::matchIdentifier => "an identifier pattern match",
            Rule::matchList => "a full list pattern match",
            Rule::matchHead => "a list head pattern match",
            Rule::matchTail => "a list tail pattern match",
            Rule::matchDict => "a non-strict dictionary pattern match",
            Rule::matchDictStrict => "a strict dictionary pattern match",
            Rule::matchDictItem => "a dictionary item pattern match",
            Rule::binding => "a variable binding",
            Rule::patternMatchBinding => "a pattern match binding",
            Rule::destructuringBiding => "a destructuring binding",
            Rule::typeDefinition => "a type definition",
            Rule::block => "a code block",
            Rule::import => "an import statement",
            Rule::importFormat => "an import format",
            Rule::importFormatText => "import as text",
            Rule::primitive => "a primitive type value",
            Rule::typeExpression => "a type expression",
            Rule::typeTerm => "a term in a type expression",
            Rule::optionalType => "an optional type",
            Rule::listType => "a list type",
            Rule::dictionaryType => "a dictionary type",
            Rule::tupleType => "a tuple type",
            Rule::recordType => "a non-strict record type",
            Rule::strictRecordType => "a strict record type",
            Rule::typeItem => "a dictionary type key-value item",
        }
    }
}

/// Parses a Ryan string and returns an abstract syntax tree (AST) object, represented by
/// its root, a [`Block`].
pub fn parse(s: &str) -> Result<Block, ParseError> {
    let mut parsed = Parser::parse(Rule::root, s).map_err(|e| ParseError {
        errors: vec![ErrorEntry::from(e).to_string_with(s)],
    })?;
    let mut error_logger = ErrorLogger::new(s);
    let main = parsed.next().expect("there is always a matching token");
    let block = if !main.as_str().is_empty() {
        Block::parse(&mut error_logger, main.into_inner())
    } else {
        Block::null()
    };

    if error_logger.errors.is_empty() {
        Ok(block)
    } else {
        Err(error_logger.into())
    }
}

#[derive(Debug)]
enum Context {
    RunningFile(Rc<str>),
    EvaluatingBinding(Rc<str>),
    SubstitutingPattern(Option<Rc<str>>),
    LoadingImport(Rc<str>),
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunningFile(filename) => write!(f, "Running {filename}"),
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
            contexts: vec![Context::RunningFile(rc_world::str_to_rc(
                environment.current_module.as_deref().unwrap_or("<main>"),
            ))],
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
