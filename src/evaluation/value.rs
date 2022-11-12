use std::fmt::Display;

use ext_php_rs::{
    convert::{FromZval, IntoZval},
    flags::DataType,
    types::Zval,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
#[derive(Debug)]
pub enum TaggedValue {
    Str(String),
    Zval(Zval),
    Usize(u64),
    Number(i64),
    Float(f64),
    Bool(bool),
}

impl Display for TaggedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{}", &s),
            Self::Usize(us) => write!(f, "{}", us),
            Self::Number(n) => write!(f, "{}", n),
            Self::Bool(true) => write!(f, "{}", 1),
            Self::Bool(false) => write!(f, ""),
            Self::Float(fl) => {
                if let Some(dec) = Decimal::from_f64(*fl) {
                    write!(f, "{}", dec.round_dp(6).normalize())
                } else {
                    write!(f, "{}", fl)
                }
            }
            Self::Zval(zv) => match zv {
                val if val.is_long() => write!(f, "{}", val.long().unwrap()),
                val if val.is_double() => write!(f, "{}", val.double().unwrap()),
                val if val.is_string() => write!(f, "{}", val.str().unwrap()),
                _ => write!(f, "{}", zv.str().unwrap_or("")),
            },
        }
    }
}

impl Clone for TaggedValue {
    fn clone(&self) -> Self {
        match self {
            Self::Str(s) => Self::Str(s.clone()),
            Self::Usize(u) => Self::Usize(*u),
            Self::Number(n) => Self::Number(*n),
            Self::Float(f) => Self::Float(*f),
            Self::Bool(b) => Self::Bool(*b),
            Self::Zval(zv) => Self::Zval(zv.shallow_clone()),
        }
    }
}

impl Default for TaggedValue {
    fn default() -> Self {
        Self::Str(String::default())
    }
}

impl From<&str> for TaggedValue {
    fn from(s: &str) -> Self {
        TaggedValue::Str(s.to_string())
    }
}

impl From<String> for TaggedValue {
    fn from(s: String) -> Self {
        TaggedValue::Str(s)
    }
}

impl From<u64> for TaggedValue {
    fn from(u: u64) -> Self {
        TaggedValue::Usize(u)
    }
}

impl From<bool> for TaggedValue {
    fn from(b: bool) -> Self {
        TaggedValue::Bool(b)
    }
}

impl FromZval<'_> for TaggedValue {
    const TYPE: ext_php_rs::flags::DataType = DataType::Mixed;
    fn from_zval(zval: &Zval) -> Option<Self> {
        Some(TaggedValue::Zval(zval.shallow_clone()))
    }
}

impl IntoZval for TaggedValue {
    const TYPE: DataType = DataType::Mixed;

    fn set_zval(self, zv: &mut Zval, persistent: bool) -> ext_php_rs::error::Result<()> {
        match self {
            Self::Str(s) => zv.set_string(&s, persistent)?,
            Self::Number(num) => zv.set_long(num),
            Self::Usize(num) => todo!("usize as long not yet possible"),
            Self::Bool(b) => zv.set_bool(b),
            Self::Float(f) => zv.set_double(f),
            Self::Zval(inner) => *zv = inner,
        };
        Ok(())
    }
}
