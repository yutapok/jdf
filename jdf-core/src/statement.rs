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

#[derive(Clone, Debug)]
pub enum Condition {
  When,
  Append,
  NoCondition,
  Unknown
}


impl Condition {
    pub fn from(s: Option<&str>) -> Self {
        match s {
          Some("WHEN") => Condition::When,
          Some("APPEND") => Condition::Append,
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
    AsteriskInArray(Asterisk),
    Nomal(String)
}

impl Expression {
    pub fn from(s: &str) -> Self {
        if s.contains("[*]") {
            Expression::AsteriskInArray(Asterisk { inner_str: s.to_string() })
        } else {
            Expression::Nomal(s.to_string())
        }
    }

    pub fn as_str(&self) -> String {
        match &*self {
            Expression::AsteriskInArray(st) => st.inner_str.clone(),
            Expression::Nomal(s) => s.to_string()
        }
    }

    pub fn is_ast(&self) -> bool {
        match *self {
          Expression::AsteriskInArray(_) => true,
          _ => false
        }
    }

    pub fn as_ast(&self) -> Option<Asterisk> {
        match &*self {
            Expression::AsteriskInArray(st) => Some(st.clone()),
            _ => None
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
            Condition::When => {
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
                    right: Some(parse_right(right))
                })
            },
            Condition::NoCondition | Condition::Append => {
              Ok(Statement {
                  select: select,
                  alias: alias.to_string(),
                  condition: cond,
                  operator: None,
                  left: None,
                  right: None
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

