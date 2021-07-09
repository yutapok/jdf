extern crate jdf_core;

use crate::Addon;

use serde_json::json;
use serde_json::{Value, Map};

#[derive(Clone)]
pub struct Sample {}

impl Addon for Sample {
    fn pipe(&self, jdf_mp: Map<String, Value>, ix: i64, v: Value) -> Value {
        let mut m = Map::new();

        m.insert("sample_key".to_string(), json!("sample_key"));
        m.insert("sample_value".to_string(), v);

        Value::Object(m)
    }
}

