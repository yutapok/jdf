use serde_json::json;
use serde_json::Value;

use crate::error;
use crate::error::JdfError;


#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
  Map,
  NoCondition,
  FlatMap,
  Unknown
}


impl Condition {
    pub fn from(s: Option<&str>) -> Self {
        match s {
          Some("MAP") => Condition::Map,
          Some("FLAT_MAP") => Condition::FlatMap,
          Some(_) => Condition::Unknown,
          None => Condition::NoCondition
        }
    }
}


#[derive(Clone, Debug)]
pub struct Statement {
    pub select: String,
    pub alias: String,
    pub condition: Condition,
    pub operator: Option<Operator>,
    pub left: Option<Value>,
    pub right: Option<Value>,
    pub addon: Option<String>
}

impl Statement {
    pub fn new(s: &str) -> Result<Self, JdfError> {
        let mut iter = s.split_whitespace();
        let select = iter.next().unwrap_or("-");
        let _as = iter.next();
        let alias = iter.next().unwrap_or(".");
        let condition = iter.next();

        let cond = Condition::from(condition);
        match cond {
            Condition::FlatMap => {
                let left = iter.next().unwrap_or("- ");
                let operator = iter.next().unwrap_or("-");
                let right = iter.next().unwrap_or(" -");

                Ok(Statement {
                    select: select.to_string(),
                    alias: alias.to_string(),
                    condition: cond,
                    operator: Some(Operator::from(operator)),
                    left: Some(parse_as(left)),
                    right: Some(parse_as(right)),
                    addon: None
                })
            },
            Condition::NoCondition => {
              Ok(Statement {
                  select: select.to_string(),
                  alias: alias.to_string(),
                  condition: cond,
                  operator: None,
                  left: None,
                  right: None,
                  addon: None
              })
            },
            Condition::Map => {
              let _ = iter.next();
              let _ = iter.next();
              let func_name = iter.next().unwrap_or("-");
              Ok(Statement {
                  select: select.to_string(),
                  alias: alias.to_string(),
                  condition: cond,
                  operator: None,
                  left: None,
                  right: None,
                  addon: Some(func_name.to_string())
              })
            },
            Condition::Unknown => Err(error!("Unexpected condition was found"))
        }
    }


}


fn parse_as(s: &str) -> Value {
    let is_number = !s.chars().map(|c| c.is_numeric()).collect::<Vec<bool>>().contains(&false);
    let is_bool = s == "true" || s == "false";

    if is_number {
      return json!(s.parse::<i64>().unwrap())
    }

    if is_bool {
      return json!(s.parse::<bool>().unwrap())
    }

    json!(s)
}

