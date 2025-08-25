use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Serialize)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        MetadataValue::String(s.to_string())
    }
}

impl Display for MetadataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataValue::String(s) => write!(f, "{s}"),
            MetadataValue::Number(n) => write!(f, "{n}"),
            MetadataValue::Integer(i) => write!(f, "{i}"),
            MetadataValue::Boolean(b) => write!(f, "{b}"),
        }
    }
}

macro_rules! impl_from_for_metadata {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for MetadataValue {
                fn from(value: $t) -> Self {
                    MetadataValue::$variant(value)
                }
            }
        )*
    };
    ($($t:ty => $variant:ident as $cast:ty),* $(,)?) => {
        $(
            impl From<$t> for MetadataValue {
                fn from(value: $t) -> Self {
                    MetadataValue::$variant(value as $cast)
                }
            }
        )*
    };
}

impl_from_for_metadata! {
    f64 => Number,
    bool => Boolean,
    i64 => Integer,
}

impl_from_for_metadata! {
    i32 => Integer as i64,
    f32 => Number as f64,
}
pub type MetadataMap = HashMap<String, MetadataValue>;
