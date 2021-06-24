use serde_json::{json, Value, Map};

pub struct Jdf {
    pub value: Value
}

impl Jdf {
    pub fn new(s: String) -> Self {
        Jdf { value: serde_json::from_str(&s).unwrap() }
    }

    pub fn convert(&mut self) ->  () {
        let default_key_str = "".to_string();
        let mp = json_flatten(default_key_str, self.value.clone());

        self.value = Value::Object(mp)
    }

    pub fn to_map(&self) -> Map<String, Value> {
        if let Value::Object(mp) = &self.value {
            mp.clone()
        } else {
            let mp: Map<String, Value> = Map::new();
            mp
        }
    }
}

fn json_flatten(key: String, value: Value) -> Map<String, Value> {
    match value {
        Value::Object(mp) =>
          mp.iter()
          .map(|(k, v)| json_flatten(format!("{}.{}",key, k), v.clone()))
          .flat_map(|fm| fm)
          .collect(),
        Value::Array(vec) => {
          vec.iter()
            .enumerate()
            .map(|(i, v)| json_flatten(format!("{}.[{}]", key, i), v.clone()))
            .flat_map(|fm| fm)
            .collect()
        },
        Value::String(s) => {
            let v: Value = serde_json::from_str(&s).unwrap_or(Value::Null);
            if v.is_object(){
                json_flatten(key.clone(), v)
            } else {
                let mut mp = Map::new();
                mp.insert(key, json!(s));
                mp
            }
        },
        others => {
            let mut mp = Map::new();
            mp.insert(key, others);
            mp
        }
    }
}
