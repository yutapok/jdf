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
  When,
  Append,
  ArrayMap,
  NoCondition,
  FlatMap,
  Unknown
}


impl Condition {
    pub fn from(s: Option<&str>) -> Self {
        match s {
          Some("WHEN") => Condition::When,
          Some("APPEND") => Condition::Append,
          Some("ARRAY_MAP") => Condition::ArrayMap,
          Some("FLAT_MAP") => Condition::FlatMap,
          Some(_) => Condition::Unknown,
          None => Condition::NoCondition
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Asterisk {
    pub inner_str: String
}

impl Asterisk {
    pub fn parse_as_vec(&self, length: usize) -> Vec<String> {
        (0..length).map(|ix| {
            self.inner_str.replace("*", &ix.to_string())
        }).collect::<Vec<String>>()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Array(String),
    Nomal(String)
}

impl Expression {
    pub fn from(s: &str) -> Self {
        if s.contains("[]") {
            Expression::Array(s.to_string())
        } else {
            Expression::Nomal(s.to_string())
        }
    }

    pub fn as_str(&self) -> String {
        match &*self {
            Expression::Array(s) => s.to_string(),
            Expression::Nomal(s) => s.to_string()
        }
    }

    pub fn is_arr(&self) -> bool {
        match *self {
            Expression::Array(_) => true,
            _ => false
        }
    }

    pub fn is_ast(&self) -> bool {
        match *self {
          Expression::Array(_) => true,
          _ => false
        }
    }


}


#[derive(Clone, Debug)]
pub struct Statement {
    pub select: Expression,
    pub alias: String,
    pub condition: Condition,
    pub operator: Option<Operator>,
    pub left: Option<Expression>,
    pub right: Option<Value>,
    pub addon: Option<String>
}

impl Statement {
    pub fn new(s: &str) -> Result<Self, JdfError> {
        let mut iter = s.split_whitespace();
        let select_s = iter.next().unwrap_or("-");
        let _as = iter.next();
        let alias = iter.next().unwrap_or(".");
        let condition = iter.next();
        let select = Expression::from(select_s);

        let cond = Condition::from(condition);
        match cond {
            Condition::When | Condition::FlatMap => {
                let left = iter.next().unwrap_or("- ");
                let operator = iter.next().unwrap_or("-");
                let right = iter.next().unwrap_or(" -");
                let left_e = Expression::from(left);

                Ok(Statement {
                    select: select,
                    alias: alias.to_string(),
                    condition: cond,
                    operator: Some(Operator::from(operator)),
                    left: Some(left_e),
                    right: Some(parse_right(right)),
                    addon: None
                })
            },
            Condition::NoCondition | Condition::Append => {
              Ok(Statement {
                  select: select,
                  alias: alias.to_string(),
                  condition: cond,
                  operator: None,
                  left: None,
                  right: None,
                  addon: None
              })
            },
            Condition::ArrayMap => {
              let _ = iter.next();
              let _ = iter.next();
              let func_name = iter.next().unwrap_or("-");
              Ok(Statement {
                  select: select,
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

    pub fn is_ast_exp(&self) -> bool {
        self.select.is_ast() && self.left.as_ref().map(|e| e.is_ast()).unwrap_or(false)
    }

}


fn parse_right(s: &str) -> Value {
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

