use std::fmt::Display;

use ext_php_rs::{types::Zval, convert::FromZval, flags::DataType};

pub enum TaggedValue {
    Str(String),
    Zval(Zval),
    Usize(u64)
}

impl Display for TaggedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{}", &s),
            Self::Usize(us) => write!(f, "{}", us),
            Self::Zval(zv) => {
                // TODO check if this behavior is ok
                write!(f, "{}", zv.str().unwrap_or(""))
            }
        }
    }
}

impl Clone for TaggedValue {
    fn clone(&self) -> Self {
        match self {
            Self::Str(s) => Self::Str(s.clone()),
            Self::Usize(u) => Self::Usize(*u),
            Self::Zval(zv) => Self::Zval(zv.shallow_clone())
        }
    }
}

impl Default for TaggedValue {
    fn default() -> Self {
        Self::Str(String::default())
    }
}

impl From<&str> for TaggedValue  {
    fn from(s: &str) -> Self {
        TaggedValue::Str(s.to_string())
    }
}

impl From<String> for TaggedValue  {
    fn from(s: String) -> Self {
        TaggedValue::Str(s)
    }
}

impl From<u64> for TaggedValue {
    fn from(u: u64) -> Self {
        TaggedValue::Usize(u)
    }
}

impl FromZval<'_> for TaggedValue {
    const TYPE: ext_php_rs::flags::DataType = DataType::Mixed;
    fn from_zval(zval: & Zval) -> Option<Self> {
        Some(TaggedValue::Zval(zval.shallow_clone()))
    }
}
