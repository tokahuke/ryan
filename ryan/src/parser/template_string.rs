use std::{fmt::Display, rc::Rc};

use indexmap::IndexMap;
use pest::iterators::Pairs;

use crate::rc_world;

use super::{value::TemplatedValue, ErrorLogger, Expression, Rule, State, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateString {
    chunks: Vec<TemplateStringChunk>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateStringChunk {
    Text(Rc<str>),
    Interpolation(Expression),
}

impl Display for TemplateString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`")?;

        for chunk in &self.chunks {
            match chunk {
                TemplateStringChunk::Interpolation(expr) => write!(f, "${{{expr}}}")?,
                TemplateStringChunk::Text(text) => {
                    for char in text.chars() {
                        match char {
                            '`' => write!(f, "\\`")?,
                            '$' => write!(f, "\\$")?,
                            ch => write!(f, "{ch}")?,
                        }
                    }
                }
            }
        }

        write!(f, "`")?;

        Ok(())
    }
}

impl TemplateString {
    pub(super) fn parse(logger: &mut ErrorLogger, pairs: Pairs<'_, Rule>) -> Self {
        let mut chunks = vec![];
        let mut chunk_builder = String::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::templateEscaped => {
                    if let Some(escaped) = pair.clone().into_inner().next() {
                        match escaped.as_rule() {
                            Rule::templateControlCode => match escaped.as_str() {
                                "`" => chunk_builder.push('`'),
                                "$" => chunk_builder.push('$'),
                                _ => unreachable!(),
                            },
                            Rule::interpolation => {
                                let chunk = rc_world::string_to_rc(chunk_builder);
                                chunk_builder = String::new();
                                chunks.push(TemplateStringChunk::Text(chunk));

                                let expression = Expression::parse(
                                    logger,
                                    pair.into_inner()
                                        .next()
                                        .expect("an interpolation always has an expression")
                                        .into_inner(),
                                );
                                chunks.push(TemplateStringChunk::Interpolation(expression));
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        chunk_builder += pair.as_str();
                    }
                }
                r => panic!("{r:?}"),
            }
        }

        if !chunk_builder.is_empty() {
            let chunk = rc_world::string_to_rc(chunk_builder);
            chunks.push(TemplateStringChunk::Text(chunk));
        }

        TemplateString { chunks }
    }

    #[must_use]
    pub(super) fn capture(
        &self,
        state: &mut State<'_>,
        provided: &mut [Rc<str>],
        values: &mut IndexMap<Rc<str>, Value>,
    ) -> Option<()> {
        for chunk in &self.chunks {
            if let TemplateStringChunk::Interpolation(expr) = chunk {
                expr.capture(state, provided, values)?;
            }
        }

        Some(())
    }

    pub(super) fn eval(&self, state: &mut State<'_>) -> Option<Value> {
        let mut builder = String::new();
        for chunk in &self.chunks {
            match chunk {
                TemplateStringChunk::Text(text) => builder += text,
                TemplateStringChunk::Interpolation(expr) => {
                    let outcome = expr.eval(state)?;
                    builder += &TemplatedValue(outcome).to_string();
                }
            }
        }

        Some(Value::Text(rc_world::string_to_rc(builder)))
    }
}
