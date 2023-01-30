use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use pest::iterators::Pairs;

use super::{expression::Expression, ErrorLogger};
use super::{Pattern, Rule, State, Value};

/// A Python-style list comprehension. This is the neares thing to `for` statement that
/// you will get in Ryan.
#[derive(Debug, Clone, PartialEq)]
pub struct ListComprehension {
    /// The expression building each item of the final list.
    pub expression: Expression,
    /// The clause matching the variables to be used in each iteration of this
    /// comprehension.
    pub for_clauses: Vec<ForClause>,
    /// An optional `if` statement that, if evaluating to false in a given iteration, will 
    /// prevent the insertion of an element in the list.
    pub if_guard: Option<IfGuard>,
}

impl Display for ListComprehension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} for {} in {}",
            self.expression, self.for_clauses[0].pattern, self.for_clauses[0].expression
        )?;

        if let Some(guard) = self.if_guard.as_ref() {
            write!(f, " if {}", guard.predicate)?;
        }

        write!(f, "]")?;

        Ok(())
    }
}

impl ListComprehension {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut expression = None;
        let mut for_clauses = vec![];
        let mut if_guard = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::expression => expression = Some(Expression::parse(logger, pair.into_inner())),
                Rule::forClause => for_clauses.push(ForClause::parse(logger, pair.into_inner())),
                Rule::ifGuard => if_guard = Some(IfGuard::parse(logger, pair.into_inner())),
                _ => unreachable!(),
            }
        }

        ListComprehension {
            expression: expression.expect("there is always an expression in a list comprehension"),
            for_clauses,
            if_guard,
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        self.expression.capture(state, provided, values)?;

        for for_clause in &self.for_clauses {
            for_clause.capture(state, provided, values)?;
        }

        if let Some(guard) = &self.if_guard {
            guard.capture(state, provided, values)?;
        }

        Some(())
    }

    /// Finicky impl: `Some(None)` is the error code; `None` skips iteration. Never
    /// use `?` here.
    pub(super) fn run_iter(
        &self,
        for_pattern: &Pattern,
        arg: Value,
        state: &mut State<'_>,
    ) -> Option<Option<Value>> {
        let mut new_bindings = HashMap::default();
        if for_pattern.bind(&arg, &mut new_bindings, state).is_none() {
            return Some(None);
        }

        let mut new_state = State {
            inherited: Some(&state),
            bindings: new_bindings,
            error: None,
            contexts: vec![],
            environment: state.environment.clone(),
        };

        let outcome = if let Some(guard) = &self.if_guard {
            guard
                .predicate
                .eval(&mut new_state)
                .and_then(|if_evalued| new_state.absorb(if_evalued.is_true()))
                .and_then(|is_true| {
                    if is_true {
                        if let Some(outcome) = self.expression.eval(&mut new_state) {
                            Some(Some(outcome))
                        } else {
                            None
                        }
                    } else {
                        Some(None)
                    }
                })
        } else {
            self.expression.eval(&mut new_state).map(Option::Some)
        };

        let maybe_error = new_state.error;

        if let Some(outcome) = outcome {
            Some(outcome)
        } else {
            state.contexts.extend(new_state.contexts);
            state.error = maybe_error;
            return None;
        }
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        if self.for_clauses.len() > 1 {
            panic!("Multiplie for-clause comprehension not yet supperted");
        }

        let iterated = match self.for_clauses[0].expression.eval(state)? {
            Value::List(list) => Value::List({
                let mut evald = vec![];

                for item in list.iter().cloned() {
                    match self.run_iter(&self.for_clauses[0].pattern, item, state) {
                        Some(Some(value)) => evald.push(value),
                        Some(None) => {}
                        None => return None,
                    }
                }

                evald.into()
            }),
            value => {
                state.raise(format!("Value `{value}` is not iterable"))?;
                return None;
            }
        };

        Some(iterated)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForClause {
    pattern: Pattern,
    expression: Expression,
}

impl ForClause {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut pattern = None;
        let mut expression = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::pattern => pattern = Some(Pattern::parse(logger, pair.into_inner())),
                Rule::expression => expression = Some(Expression::parse(logger, pair.into_inner())),
                _ => unreachable!(),
            }
        }

        ForClause {
            pattern: pattern.expect("there is always a pattern in a for clause"),
            expression: expression.expect("there is always an expression in a for clause"),
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        self.expression.capture(state, provided, values)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfGuard {
    predicate: Expression,
}

impl IfGuard {
    pub(super) fn parse(logger: &mut ErrorLogger, mut pairs: Pairs<'_, Rule>) -> Self {
        let predicate = Expression::parse(
            logger,
            pairs
                .next()
                .expect("there is always a predicate in an if guard")
                .into_inner(),
        );

        IfGuard { predicate }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        self.predicate.capture(state, provided, values)
    }
}
