use pest::iterators::Pairs;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use super::binding::Binding;
use super::expression::Expression;
use super::literal::Literal;
use super::value::Value;
use super::ErrorLogger;
use super::Rule;
use super::State;

/// A block of Ryan code. This consists of a list of statements and a return expression at the end.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    /// The list of bindings to be applied and evaluated before running the final expression.
    pub bindings: Vec<Binding>,
    /// The expression that will build the final outcome of this block.
    pub expression: Expression,
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for binding in &self.bindings {
            writeln!(f, "{binding}")?;
        }

        write!(f, "{}", self.expression)?;

        Ok(())
    }
}

impl Block {
    /// Creates an empty block that returns `null`. This is the default value for an empty
    /// Ryan program.
    pub fn null() -> Block {
        Block {
            bindings: vec![],
            expression: Expression::Literal(Literal::Null),
        }
    }
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut bindings = vec![];
        let mut expression = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::binding => bindings.push(Binding::parse(logger, pair.into_inner())),
                Rule::expression => expression = Some(Expression::parse(logger, pair.into_inner())),
                _ => unreachable!(),
            }
        }

        Block {
            bindings,
            expression: expression.unwrap_or(Expression::Literal(Literal::Null)),
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        let mut provided = provided.to_vec();

        for binding in &self.bindings {
            binding.capture(state, &mut provided, values)?;
        }

        self.expression.capture(state, &mut provided, values)?;

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        for binding in &self.bindings {
            binding.eval(state)?;
        }

        self.expression.eval(state)
    }
}
