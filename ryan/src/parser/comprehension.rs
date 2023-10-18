use std::fmt::Display;
use std::rc::Rc;

use indexmap::IndexMap;
use pest::iterators::Pairs;

use super::{expression::Expression, ErrorLogger};
use super::{Pattern, Rule, State, Value};

/// A Python-style list comprehension. This is the nearest thing to `for` statement that
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
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        let mut provided = provided.to_vec();

        for for_clause in &self.for_clauses {
            for_clause.capture(state, &mut provided, values)?;
        }

        if let Some(guard) = &self.if_guard {
            guard.capture(state, &mut *provided, values)?;
        }

        self.expression.capture(state, &mut *provided, values)?;

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        let mut bag = vec![];
        self.run_iter(state, &mut bag, &self.for_clauses)?;

        Some(Value::List(bag.into()))
    }

    fn run_iter(
        &self,
        state: &mut State<'_>,
        bag: &mut Vec<Value>,
        for_clauses: &[ForClause],
    ) -> Option<()> {
        let for_clause = &for_clauses[0];
        let iterable = for_clause.expression.eval(state)?;
        let iter = match iterable.iter() {
            Ok(iter) => iter,
            Err(err) => {
                state.raise(err)?;
                return None;
            }
        };

        if for_clauses.len() > 1 {
            // Recurse
            for item in iter {
                let new_bindings = for_clause.bindings(state, &item)?;
                let mut new_state = state.new_local(new_bindings);

                self.run_iter(&mut new_state, bag, &for_clauses[1..])?;
            }
        } else {
            // Loop
            for item in iter {
                let new_bindings = for_clause.bindings(state, &item)?;
                let mut new_state = state.new_local(new_bindings);

                if let Some(guard) = &self.if_guard {
                    guard.maybe_eval(&mut new_state, |s| {
                        let value = self.expression.eval(s)?;
                        bag.push(value);
                        Some(())
                    })?;
                } else {
                    let value = self.expression.eval(&mut new_state)?;
                    bag.push(value);
                }
            }
        }

        Some(())
    }
}

/// A Python-style dictionary comprehension. This is the nearest thing to `for` statement that
/// you will get in Ryan.
#[derive(Debug, Clone, PartialEq)]
pub struct DictComprehension {
    /// The expression building each item of the final dictionary.
    pub key_value_clause: KeyValueClause,
    /// The clause matching the variables to be used in each iteration of this
    /// comprehension.
    pub for_clauses: Vec<ForClause>,
    /// An optional `if` statement that, if evaluating to false in a given iteration, will
    /// prevent the insertion of an element in the dictionary.
    pub if_guard: Option<IfGuard>,
}

impl Display for DictComprehension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{} for {} in {}",
            self.key_value_clause, self.for_clauses[0].pattern, self.for_clauses[0].expression
        )?;

        if let Some(guard) = self.if_guard.as_ref() {
            write!(f, " if {}", guard.predicate)?;
        }

        write!(f, "}}")?;

        Ok(())
    }
}

impl DictComprehension {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut key_value_clause = None;
        let mut for_clauses = vec![];
        let mut if_guard = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::keyValueClause => {
                    key_value_clause = Some(KeyValueClause::parse(logger, pair.into_inner()))
                }
                Rule::forClause => for_clauses.push(ForClause::parse(logger, pair.into_inner())),
                Rule::ifGuard => if_guard = Some(IfGuard::parse(logger, pair.into_inner())),
                _ => unreachable!(),
            }
        }

        DictComprehension {
            key_value_clause: key_value_clause
                .expect("there is always an expression in a list comprehension"),
            for_clauses,
            if_guard,
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        let mut provided = provided.to_vec();

        for for_clause in &self.for_clauses {
            for_clause.capture(state, &mut provided, values)?;
        }

        if let Some(guard) = &self.if_guard {
            guard.capture(state, &mut *provided, values)?;
        }

        self.key_value_clause
            .capture(state, &mut *provided, values)?;

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        let mut bag = IndexMap::new();
        self.run_iter(state, &mut bag, &self.for_clauses)?;

        Some(Value::Map(bag.into()))
    }

    fn run_iter(
        &self,
        state: &mut State<'_>,
        bag: &mut IndexMap<Rc<str>, Value>,
        for_clauses: &[ForClause],
    ) -> Option<()> {
        let for_clause = &for_clauses[0];
        let iterable = for_clause.expression.eval(state)?;
        let iter = match iterable.iter() {
            Ok(iter) => iter,
            Err(err) => {
                state.raise(err)?;
                return None;
            }
        };

        if for_clauses.len() > 1 {
            // Recurse
            for item in iter {
                let new_bindings = for_clause.bindings(state, &item)?;
                let mut new_state = state.new_local(new_bindings);
                self.run_iter(&mut new_state, bag, &for_clauses[1..])?;
            }
        } else {
            // Loop
            for item in iter {
                let new_bindings = for_clause.bindings(state, &item)?;
                let mut new_state = state.new_local(new_bindings);

                if let Some(guard) = &self.if_guard {
                    guard.maybe_eval(&mut new_state, |s| {
                        let (key, value) = self.key_value_clause.eval(s)?;
                        bag.insert(key, value);
                        Some(())
                    })?;
                } else {
                    let (key, value) = self.key_value_clause.eval(&mut new_state)?;
                    bag.insert(key, value);
                }
            }
        }

        Some(())
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
        provided: &mut Vec<Rc<str>>,
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        self.expression.capture(state, provided, values)?;
        self.pattern.capture(state, provided, values)?;
        self.pattern.provided(provided);

        Some(())
    }

    fn bindings(&self, state: &mut State<'_>, value: &Value) -> Option<IndexMap<Rc<str>, Value>> {
        let mut new_bindings = IndexMap::new();
        let bind = self.pattern.bind(&value, &mut new_bindings, state)?;
        state.absorb(bind)?;

        Some(new_bindings)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueClause {
    key: Expression,
    value: Expression,
}

impl Display for KeyValueClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

impl KeyValueClause {
    pub(super) fn parse(logger: &mut ErrorLogger, mut pairs: Pairs<'_, Rule>) -> Self {
        let key = Expression::parse(
            logger,
            pairs
                .next()
                .expect("there is always a key in a key-value comprehension clause")
                .into_inner(),
        );
        let value = Expression::parse(
            logger,
            pairs
                .next()
                .expect("there is always a key in a key-value comprehension clause")
                .into_inner(),
        );

        KeyValueClause { key, value }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        self.key.capture(state, provided, values)?;
        self.value.capture(state, provided, values)?;
        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<(Rc<str>, Value)> {
        let key = self.key.eval(state)?;
        let Value::Text(key) = key else {
            state.raise(format!("Dict comprehension received non-text value {key}"));
            return None;
        };
        let value = self.value.eval(state)?;
        Some((key, value))
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
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        self.predicate.capture(state, provided, values)
    }

    fn maybe_eval<F>(&self, state: &mut State<'_>, f: F) -> Option<()>
    where
        F: FnOnce(&mut State<'_>) -> Option<()>,
    {
        let truthiness = self.predicate.eval(state)?.is_true();
        if state.absorb(truthiness)? {
            f(state)?;
        }

        Some(())
    }
}
