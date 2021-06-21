use std::sync::Arc;
use serde_json::{Value, Map};

use crate::query::Statement;


#[derive(Debug)]
pub enum Operator {
  EQ,
  NEQ,
  Unknown
}

impl Operator {
  pub fn from(s: &str) -> Self {
    match s {
      "==" => Operator::EQ,
      "!=" => Operator::NEQ,
      _ => Operator::Unknown
    }
  }
}

struct Search <'a> {
    pub is_asterisk: bool,
    mp: &'a Arc<Map<String, Value>>,
    s: &'a str
}

impl<'a> Search<'a> {
    pub fn new(s: &'a str, mp: &'a Arc<Map<String, Value>>) -> Self {
      Search {
        is_asterisk: s.contains("[*]"),
        mp: &mp,
        s: s
      }
    }

    pub fn results(&self) -> Option<Value> {
      if self.is_asterisk {
          let mut v = Vec::new();
          for i in 0..self.mp.len() {
              let select = self.s.replace("*", &i.to_string());
              let result = self.mp.get(&select);
              v.push(result.map(|v| v.clone()).unwrap_or(Value::Null));
          }
          Some(Value::Array(v))
      } else {
          self.mp.get(self.s).map(|v| v.clone())
      }
    }
}

#[derive(Debug)]
pub enum Condition {
  When,
  Append,
  NoCondition,
  Unknown
}


impl Condition {
    pub fn from(s: &str) -> Self {
        match s {
          "WHEN" => Condition::When,
          "APPEND" => Condition::Append,
          _ => Condition::Unknown
        }
    }

    pub fn evaluate(&self, mp: &Arc<Map<String, Value>>, stmt: &Statement) -> Value {
        match *self {
            Condition::NoCondition =>  {
                mp.get(&stmt.select.as_str()).unwrap_or(&Value::Null).clone()
            },
            Condition::When => Value::Object(self.when(mp, stmt)),
            Condition::Append => Value::Array(self.append(mp, stmt.select.as_str())
              .iter()
              .map(|s| Value::String(s.to_string()))
              .collect::<Vec<Value>>()
            ),
            Condition::Unknown => Value::Null
        }
    }

    fn when(&self, mp: &Arc<Map<String, Value>>, stmt: &Statement) -> Map<String, Value> {
        //TODO: refactor
        //let mut ret_mp = Map::new();
        //if stmt.left.is_some() && stmt.left.as_ref().unwrap().is_ast() && stmt.select.is_ast() {
        //    let left = stmt.left.as_ref().map(|op| op.as_str()).unwrap_or("- ".to_string());
        //    let select = stmt.left.as_ref().map(|op| op.as_str()).unwrap_or("- ".to_string());
        //    for i in 0..mp.iter().len() {
        //        let left_s = &left.replace("*", &i.to_string());
        //        let select_s = &select.replace("*", &i.to_string());
        //        let select_v = mp.get(&select).unwrap_or(&Value::Null);
        //        let left_vs = extract_as_str(mp, left_s.to_string()).unwrap_or("- ".to_string());
        //        let right_vs = stmt.right.as_ref().unwrap_or(&" -".to_string());
        //        if self.when_calc(left_vs, stmt.operator.unwrap_or(Operator::Unknown), right_vs.to_string()){
        //            ret_mp.insert(stmt.alias, select_v.clone());
        //        }
        //    }
        //} else {
        //    let select = mp.get(&stmt.select.as_str()).unwrap_or(&Value::Null);
        //    let left = extract_as_str(
        //      mp,
        //      stmt.left
        //        .map(|o| o.as_str())
        //        .unwrap_or("- ".to_string())
        //    ).unwrap_or("- ".to_string());
        //    let right = stmt.right.unwrap_or(" -".to_string());

        //    if self.when_calc(left, stmt.operator.unwrap_or(Operator::Unknown), right){
        //        ret_mp.insert(stmt.alias, select.clone());
        //    }
        //}

        //ret_mp
    }

    fn when_calc(&self, left: String, operator: Operator, right: String) -> bool {
        match operator {
          Operator::EQ => left == right,
          Operator::NEQ => left != right,
          _ => false
        }
    }

    fn append(&self, mp: &Arc<Map<String, Value>>, select: String) ->  Vec<String> {
        let search = Search::new(&select, mp);
        if !search.is_asterisk {
            let v_ = Vec::with_capacity(1);
            return v_
        }

        search.results().iter()
          .filter(|v| **v != Value::Null)
          .map(|k| match k {
              Value::String(s) => s.to_string(),
              _ => "".to_string()
          })
          .filter(|opt| !opt.is_empty())
          .collect::<Vec<String>>()
    }
}

fn extract_as_str(mp: &Arc<Map<String, Value>>, s: String) ->  Option<String> {
    let v = mp.get(&s).unwrap_or(&Value::Null);
    match v {
        &Value::String(s) => Some(s),
        _ => None
    }
}

