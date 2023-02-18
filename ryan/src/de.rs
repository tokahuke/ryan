use std::borrow::Cow;
use std::fmt::Display;

use serde::de::value::{MapAccessDeserializer, MapDeserializer, SeqDeserializer, StrDeserializer};
use serde::de::{IntoDeserializer, Visitor};
use serde::Deserializer;

use crate::parser::{Type, Value};

#[derive(Debug, Clone, Copy)]
pub enum MaterializedType {
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    String,
    Bytes,
    List,
    Map,
    Enum,
}

impl Display for MaterializedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MaterializedType::Unit => "null",
            MaterializedType::Bool => "a boolean",
            MaterializedType::I8 => "an 8-bit signed integer",
            MaterializedType::I16 => "a 16-bit signed integer",
            MaterializedType::I32 => "a 32-bit signed integer",
            MaterializedType::I64 => "a 64-bit signed integer",
            MaterializedType::U8 => "an 8-bit positive integer",
            MaterializedType::U16 => "a 16-bit positive integer",
            MaterializedType::U32 => "a 32-bit positive integer",
            MaterializedType::U64 => "a 64-bit positive integer",
            MaterializedType::F32 => "a single precision float",
            MaterializedType::F64 => "a double precision float",
            MaterializedType::Char => "a single character",
            MaterializedType::String => "text",
            MaterializedType::Bytes => "a list of 8-bit unsigned integers",
            MaterializedType::List => "a list of values",
            MaterializedType::Map => "a key-value map",
            MaterializedType::Enum => "an enumeration",
        };

        write!(f, "{name}")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("{0}")]
    Message(String),
    #[error("Cannot dynamically deserialize value of type {typ}")]
    DeserializeAnyError { typ: Type },
    #[error("expected {expected} but got value of type {got}")]
    TypeError {
        expected: MaterializedType,
        got: Type,
    },
    #[error("cannot fit the number {got} into {expected}")]
    RangeError {
        expected: MaterializedType,
        got: i64,
    },
    #[error("expected list of length {expected} but got list of length {got}")]
    LengthMismatch { expected: usize, got: usize },
}

impl serde::de::Error for DecodeError {
    fn custom<T: Display>(msg: T) -> Self {
        DecodeError::Message(msg.to_string())
    }
}

pub struct RyanDeserializer<'de> {
    pub(crate) value: Cow<'de, Value>,
}

impl<'de> IntoDeserializer<'de, DecodeError> for RyanDeserializer<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for RyanDeserializer<'de> {
    type Error = DecodeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Null => self.deserialize_unit(visitor),
            Value::Bool(_) => self.deserialize_bool(visitor),
            Value::Integer(_) => self.deserialize_i64(visitor),
            Value::Float(_) => self.deserialize_f64(visitor),
            Value::Text(_) => self.deserialize_str(visitor),
            Value::List(_) => self.deserialize_seq(visitor),
            Value::Map(_) => self.deserialize_map(visitor),
            v => Err(DecodeError::DeserializeAnyError {
                typ: v.canonical_type(),
            }),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Bool(b) => visitor.visit_bool(b),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::Bool,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as i8 as i64 == int => visitor.visit_i8(int as i8),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::I8,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::I8,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as i16 as i64 == int => visitor.visit_i16(int as i16),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::I16,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::I16,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as i32 as i64 == int => visitor.visit_i32(int as i32),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::I32,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::I32,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) => visitor.visit_i64(int),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::I64,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as u8 as i64 == int => visitor.visit_u8(int as u8),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::U8,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::U8,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as u16 as i64 == int => visitor.visit_u16(int as u16),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::U16,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::U16,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as u32 as i64 == int => visitor.visit_u32(int as u32),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::U32,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::U32,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as u64 as i64 == int => visitor.visit_u64(int as u64),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::U64,
                got: int,
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::U64,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as f32 as i64 == int => visitor.visit_f32(int as f32),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::F32,
                got: int,
            }),
            &Value::Float(float) => visitor.visit_f32(float as f32),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::F32,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            &Value::Integer(int) if int as f64 as i64 == int => visitor.visit_f64(int as f64),
            &Value::Integer(int) => Err(DecodeError::RangeError {
                expected: MaterializedType::F64,
                got: int,
            }),
            &Value::Float(float) => visitor.visit_f64(float),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::F64,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Text(s) if s.len() == 1 => {
                visitor.visit_char(s.chars().next().expect("non-empty strings"))
            }
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::Char,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Text(s) => visitor.visit_str(s),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::String,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::List(list) => {
                let bytes = list
                    .iter()
                    .map(|item| match item {
                        &Value::Integer(int) if int as u8 as i64 == int => Ok(int as u8),
                        &Value::Integer(int) => Err(DecodeError::RangeError {
                            expected: MaterializedType::U8,
                            got: int,
                        }),
                        v => Err(DecodeError::TypeError {
                            expected: MaterializedType::U8,
                            got: v.canonical_type(),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                visitor.visit_byte_buf(bytes)
            }
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::Bytes,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(Self { value: self.value }),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Null => visitor.visit_unit(),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::Unit,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::List(list) => {
                let values = list.iter().map(|item| Self {
                    value: Cow::Owned(item.clone()),
                });
                visitor.visit_seq(SeqDeserializer::new(values))
            }
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::List,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::List(list) if list.len() == len => self.deserialize_seq(visitor),
            Value::List(list) => Err(DecodeError::LengthMismatch {
                expected: len,
                got: list.len(),
            }),
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::List,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Map(dict) => {
                let values = dict.iter().map(|(key, item)| {
                    (
                        Self {
                            value: Cow::Owned(Value::Text(key.clone())),
                        },
                        Self {
                            value: Cow::Owned(item.clone()),
                        },
                    )
                });
                visitor.visit_map(MapDeserializer::new(values))
            }
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::Map,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &*self.value {
            Value::Text(string) => visitor.visit_enum(StrDeserializer::new(string)),
            Value::Map(dict) => {
                let values = dict.iter().map(|(key, item)| {
                    (
                        Self {
                            value: Cow::Owned(Value::Text(key.clone())),
                        },
                        Self {
                            value: Cow::Owned(item.clone()),
                        },
                    )
                });
                visitor.visit_enum(MapAccessDeserializer::new(MapDeserializer::new(values)))
            }
            v => Err(DecodeError::TypeError {
                expected: MaterializedType::Enum,
                got: v.canonical_type(),
            }),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
