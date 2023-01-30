use pest::{
    iterators::Pairs,
    pratt_parser::{Op, PrattParser},
};
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{rc_world, utils::QuotedStr};

use super::ErrorLogger;
use super::Rule;
use super::State;
use super::{comprehension::ListComprehension, operation::BinaryOperator};
use super::{import::Import, operation::BinaryOperation};
use super::{
    literal::Literal,
    operation::{PrefixOperation, PrefixOperator},
};
use super::{
    operation::{PostfixOperation, PostfixOperator},
    value::Value,
};

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::Assoc::*;

        PrattParser::new()
            .op(Op::infix(Rule::orOp, Left))
            .op(Op::infix(Rule::andOp, Left))
            .op(Op::prefix(Rule::notOp))
            .op(
                Op::infix(Rule::equalsOp, Left)
                | Op::infix(Rule::notEqualsOp, Left)
                | Op::infix(Rule::typeMatchesOp, Left)
                | Op::infix(Rule::greaterOp, Left)
                | Op::infix(Rule::greaterEqualOp, Left)
                | Op::infix(Rule::lesserOp, Left)
                | Op::infix(Rule::lesserEqualOp, Left)
            )
            .op(Op::infix(Rule::plusOp, Left) | Op::infix(Rule::minusOp, Left))
            .op(Op::infix(Rule::remainderOp, Left))
            .op(Op::infix(Rule::timesOp, Left) | Op::infix(Rule::dividedOp, Left))
            .op(Op::infix(Rule::defaultOp, Left))
            .op(Op::infix(Rule::juxtapositionOp, Right))
            .op(Op::postfix(Rule::accessOp))
    };
}

/// Transformations of Ryan values.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Builds a list of Ryan values.
    List(Vec<Expression>),
    /// Builds a dictionary of Ryan values.
    Dict(Dict),
    /// Based on an expressing returning a `bool`, executes either of the supplied
    /// expressions.
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
    /// Builds a Ryan value from a litteral.
    Literal(Literal),
    /// Builds a Ryan value of a binary operation over two Ryan values.
    BinaryOperation(Box<BinaryOperation>),
    /// Builds a Ryan value of a prefix operator applied on a Ryan value.
    PrefixOperation(Box<PrefixOperation>),
    /// Builds a Ryan value of a postfix operator applied on a Ryan value.
    PostfixOperation(Box<PostfixOperation>),
    /// Produces a Ryan value from an `import` statement.
    Import(Import),
    /// Creates a Ryan value from a list comprehension.
    ListComprehension(Box<ListComprehension>),
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Literal(Literal::default())
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(list) => {
                write!(f, "[")?;
                crate::utils::fmt_list(f, list)?;
                write!(f, "]")?;
            }
            Self::Dict(dict) => {
                write!(f, "{{")?;
                crate::utils::fmt_list(f, &dict.items)?;
                write!(f, "}}")?;
            }
            Self::Literal(lit) => write!(f, "{lit}")?,
            Self::BinaryOperation(op) => write!(f, "{op}")?,
            Self::PrefixOperation(op) => write!(f, "{op}")?,
            Self::PostfixOperation(op) => write!(f, "{op}")?,
            Self::Conditional(r#if, then, r#else) => {
                write!(f, "if {} then {} else {}", r#if, r#then, r#else)?
            }
            Self::Import(import) => write!(f, "{import}")?,
            Self::ListComprehension(comprehension) => write!(f, "{comprehension}")?,
        }

        Ok(())
    }
}

impl Expression {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let logger_cell = Rc::new(RefCell::new(logger));
        let logger_cell_posfix = logger_cell.clone();

        PRATT_PARSER
            .map_primary(|pair| match pair.as_rule() {
                Rule::list => {
                    let exprs = pair
                        .into_inner()
                        .map(|pair| Expression::parse(*logger_cell.borrow_mut(), pair.into_inner()))
                        .collect::<Vec<_>>();
                    Expression::List(exprs)
                }
                Rule::dict => {
                    Expression::Dict(Dict::parse(*logger_cell.borrow_mut(), pair.into_inner()))
                }
                Rule::conditional => {
                    let mut pairs = pair.into_inner();
                    let mut next = || {
                        Expression::parse(
                            *logger_cell.borrow_mut(),
                            pairs
                                .next()
                                .expect("clause in conditional was expected")
                                .into_inner(),
                        )
                    };
                    Expression::Conditional(Box::new(next()), Box::new(next()), Box::new(next()))
                }
                Rule::literal => Expression::Literal(Literal::parse(
                    *logger_cell.borrow_mut(),
                    pair.into_inner(),
                )),
                Rule::import => {
                    Expression::Import(Import::parse(*logger_cell.borrow_mut(), pair.into_inner()))
                }
                Rule::expression => Self::parse(*logger_cell.borrow_mut(), pair.into_inner()),
                Rule::listComprehension => Expression::ListComprehension(Box::new(
                    ListComprehension::parse(*logger_cell.borrow_mut(), pair.into_inner()),
                )),
                _ => unreachable!(),
            })
            .map_infix(|left, op, right| {
                Expression::BinaryOperation(Box::new(BinaryOperation {
                    left,
                    op: BinaryOperator::parse(op),
                    right,
                }))
            })
            .map_prefix(|op, right| {
                Expression::PrefixOperation(Box::new(PrefixOperation {
                    op: PrefixOperator::parse(op),
                    right,
                }))
            })
            .map_postfix(move |left, op| {
                Expression::PostfixOperation(Box::new(PostfixOperation {
                    op: PostfixOperator::parse(*logger_cell_posfix.borrow_mut(), op),
                    left,
                }))
            })
            .parse(pairs)
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut HashMap<Rc<str>, Value>,
    ) -> Option<()> {
        match self {
            Self::List(list) => {
                for item in list {
                    item.capture(state, provided, values)?;
                }
            }
            Self::Dict(dict) => {
                for item in &dict.items {
                    item.value.capture(state, provided, values)?;
                    if let Some(g) = &item.guard {
                        g.capture(state, provided, values)?;
                    }
                }
            }
            Self::Conditional(r#if, then, r#else) => {
                r#if.capture(state, provided, values)?;
                then.capture(state, provided, values)?;
                r#else.capture(state, provided, values)?;
            }
            Self::Literal(lit) => {
                lit.capture(state, provided, values)?;
            }
            Self::BinaryOperation(op) => {
                op.left.capture(state, provided, values)?;
                op.right.capture(state, provided, values)?;
            }
            Self::PrefixOperation(op) => op.right.capture(state, provided, values)?,
            Self::PostfixOperation(op) => op.left.capture(state, provided, values)?,
            Self::Import(_) => {}
            Self::ListComprehension(comprehension) => {
                comprehension.capture(state, provided, values)?
            }
        };

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        let returned = match self {
            Self::List(list) => Value::List(Rc::from(
                list.iter()
                    .map(|expr| expr.eval(state))
                    .collect::<Option<Vec<_>>>()?,
            )),
            Self::Dict(dict) => Value::Map(Rc::new({
                let mut evald = HashMap::new();
                for item in &dict.items {
                    if let Some(g) = &item.guard {
                        let tested = g.eval(state)?.is_true();
                        if !state.absorb(tested)? {
                            continue;
                        }
                    }

                    evald.insert(rc_world::str_to_rc(&item.key), item.value.eval(state)?);
                }
                evald
            })),
            Self::Conditional(r#if, then, r#else) => {
                let if_evalued = r#if.eval(state)?;
                let to_eval = if state.absorb(if_evalued.is_true())? {
                    then
                } else {
                    r#else
                };

                to_eval.eval(state)?
            }
            Self::Literal(lit) => lit.eval(state)?,
            Self::BinaryOperation(op) => op.eval(state)?,
            Self::PrefixOperation(op) => op.eval(state)?,
            Self::PostfixOperation(op) => op.eval(state)?,
            Self::Import(import) => import.eval(state)?,
            Self::ListComprehension(comprehension) => comprehension.eval(state)?,
        };

        Some(returned)
    }
}

/// An association of string values to Ryan values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dict {
    /// The entries of this association.
    pub items: Vec<DictItem>,
}

impl Dict {
    fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut items = vec![];

        for pair in pairs {
            match pair.as_rule() {
                Rule::dictItem => items.push(DictItem::parse(logger, pair.into_inner())),
                _ => unreachable!(),
            }
        }

        Dict { items }
    }
}

/// An entry of a dictionary expression.
#[derive(Debug, Clone, PartialEq)]
pub struct DictItem {
    /// The string value associated with the Ryan value.
    pub key: Rc<str>,
    /// The expression that evaluates to the value of this association.
    pub value: Expression,
    /// An optional `if` guard. If the supplied expression evaluates to `false`, the
    /// current key-value pair is not inserted in the final dictionary.
    pub guard: Option<Expression>,
}

impl Display for DictItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(g) = &self.guard {
            write!(f, "{}: {} if {}", QuotedStr(&self.key), self.value, g)
        } else {
            write!(f, "{}: {}", QuotedStr(&self.key), self.value)
        }
    }
}

impl DictItem {
    fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut key = None;
        let mut value = None;
        let mut guard = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::identifier => key = Some(rc_world::str_to_rc(pair.as_str())),
                Rule::text => {
                    key = Some(rc_world::string_to_rc(
                        logger.absorb(&pair, snailquote::unescape(pair.as_str())),
                    ));
                }
                Rule::expression => value = Some(Expression::parse(logger, pair.into_inner())),
                Rule::ifGuard => {
                    guard = Some(Expression::parse(
                        logger,
                        pair.into_inner()
                            .next()
                            .expect("there is always an expression in an if guard")
                            .into_inner(),
                    ))
                }
                _ => unreachable!(),
            }
        }

        DictItem {
            key: key.expect("there is always a key in dict item"),
            value: value.expect("there is always a value in dict item"),
            guard,
        }
    }
}
