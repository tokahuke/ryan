use pest::iterators::Pairs;
use std::fmt::Display;
use std::{collections::HashMap, rc::Rc};

use crate::rc_world;

use super::block::Block;
use super::pattern::Pattern;
use super::types::TypeExpression;
use super::value::PatternMatch;
use super::ErrorLogger;
use super::Rule;
use super::State;
use super::{Context, Value};

/// A binding is a `let ... = ...;` or a `type ... = ...;` statement that creates new
/// variables, types and patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    /// Defines a new pattern or a new rule for an existing pattern.
    PatternMatchDefinition {
        /// The identifier for the pattern.
        identifier: Rc<str>,
        /// The pattern against whitch to match the input.
        pattern: Pattern,
        /// The code to be executed if the pattern is satisfied.
        block: Block,
    },
    /// A destructuring match that binds the variables provided by the pattern to the
    /// corresponding bits and pieces returned by the block of code.
    Destructuring {
        /// The pattern against which to bind the result of the block.
        pattern: Pattern,
        /// The block to be executed to produce the value against which the pattern will
        /// be matched.
        block: Block,
    },
    /// A type definition. This binds an identifier to a type value.
    TypeDefinition {
        /// The name of the type.
        identifier: Rc<str>,
        /// The expression defining this type.
        type_expression: TypeExpression,
    },
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatternMatchDefinition {
                identifier,
                pattern,
                block,
            } => {
                if block.bindings.is_empty() {
                    write!(f, "let {identifier} {pattern} = {block}")?;
                } else {
                    // Indent:
                    let blockstr = block.to_string().replace('\n', "\n    ");
                    write!(f, "let {identifier} {pattern} =\n    {blockstr};")?;
                }
            }
            Self::Destructuring { pattern, block } => {
                if block.bindings.is_empty() {
                    write!(f, "let {pattern} = {block}")?;
                } else {
                    // Indent:
                    let blockstr = block.to_string().replace('\n', "\n    ");
                    write!(f, "let {pattern} =\n    {blockstr};")?;
                }
            }
            Self::TypeDefinition {
                identifier,
                type_expression,
            } => {
                write!(f, "type {identifier} = {type_expression};")?;
            }
        }

        Ok(())
    }
}

impl Binding {
    pub(super) fn parse(logger: &mut ErrorLogger, mut pairs: Pairs<'_, Rule>) -> Self {
        let pair = pairs
            .next()
            .expect("there is always a binding type in a binding");

        match pair.as_rule() {
            Rule::patternMatchBinding => {
                let mut identifier = None;
                let mut pattern = None;
                let mut block = None;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::identifier => identifier = Some(rc_world::str_to_rc(pair.as_str())),
                        Rule::pattern => pattern = Some(Pattern::parse(logger, pair.into_inner())),
                        Rule::block => block = Some(Block::parse(logger, pair.into_inner())),
                        _ => unreachable!(),
                    }
                }

                Binding::PatternMatchDefinition {
                    identifier: identifier
                        .expect("tere is always an identifier in a pattern match definition"),
                    pattern: pattern.expect("there is always a pattern in a pattern definition"),
                    block: block.expect("there is always an expression in a pattern definition"),
                }
            }
            Rule::destructuringBiding => {
                let mut pattern = None;
                let mut block = None;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::pattern => pattern = Some(Pattern::parse(logger, pair.into_inner())),
                        Rule::block => block = Some(Block::parse(logger, pair.into_inner())),
                        _ => unreachable!(),
                    }
                }

                Binding::Destructuring {
                    pattern: pattern.expect("there is always a pattern in a destructuring binding"),
                    block: block.expect("there is always an expression in a destructuring binding"),
                }
            }
            Rule::typeDefinition => {
                let mut identifier = None;
                let mut type_expression = None;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::identifier => identifier = Some(rc_world::str_to_rc(pair.as_str())),
                        Rule::typeExpression => {
                            type_expression = Some(TypeExpression::parse(logger, pair.into_inner()))
                        }
                        _ => unreachable!(),
                    }
                }

                Binding::TypeDefinition {
                    identifier: identifier
                        .expect("there is always an identifier in a type definition"),
                    type_expression: type_expression
                        .expect("there is always an expression in a type definition"),
                }
            }
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut Vec<Rc<str>>,
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        match self {
            Self::PatternMatchDefinition {
                identifier,
                pattern,
                block,
            } => {
                pattern.capture(state, provided, values)?;
                pattern.provided(provided);
                provided.push(identifier.clone());
                block.capture(state, provided, values)?;
            }
            Self::Destructuring { pattern, block } => {
                pattern.capture(state, provided, values)?;
                pattern.provided(provided);
                block.capture(state, provided, values)?;
            }
            Self::TypeDefinition {
                identifier,
                type_expression,
            } => {
                provided.push(identifier.clone());
                type_expression.capture(state, provided, values)?;
            }
        }

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<()> {
        match self {
            Self::PatternMatchDefinition {
                identifier,
                pattern,
                block,
            } => {
                state.push_ctx(Context::EvaluatingBinding(identifier.clone()));

                let mut provided = vec![];
                pattern.provided(&mut provided);

                let mut captured = HashMap::default();
                block.capture(state, &mut provided, &mut captured)?;

                if let Some(Value::PatternMatches(_, mut matches)) =
                    state.bindings.remove(identifier)
                {
                    // Insert new alternative:
                    matches.push(Rc::new(PatternMatch {
                        captures: captured,
                        pattern: pattern.clone(),
                        block: block.clone(),
                    }));
                    // Reinsert value into the bindings;
                    state.bindings.insert(
                        identifier.clone(),
                        Value::PatternMatches(identifier.clone(), matches),
                    );
                } else {
                    state.bindings.insert(
                        identifier.clone(),
                        Value::PatternMatches(
                            identifier.clone(),
                            vec![Rc::new(PatternMatch {
                                captures: captured,
                                pattern: pattern.clone(),
                                block: block.clone(),
                            })],
                        ),
                    );
                }

                state.pop_ctx();
            }
            Self::Destructuring { pattern, block } => {
                let evaluated = block.eval(state)?;
                let mut new_bindings = HashMap::default();

                if let Err(err) = pattern.bind(&evaluated, &mut new_bindings, state)? {
                    state.raise(format!("{err}"))?;
                    return None;
                }

                state.bindings.extend(new_bindings);
            }
            Self::TypeDefinition {
                identifier,
                type_expression,
            } => {
                let resolved_type = type_expression.eval(state)?;
                state
                    .bindings
                    .insert(identifier.clone(), Value::Type(resolved_type));
            }
        }

        Some(())
    }
}
