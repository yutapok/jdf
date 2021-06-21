use serde_json::{Result, Value, Map};

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

    pub fn dumps(&mut self) -> Result<String> {
        serde_json::to_string(&self.convert())
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
        others => {
            let mut mp = Map::new();
            mp.insert(key, others);
            mp
        }
    }
}
