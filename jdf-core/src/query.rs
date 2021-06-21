use crate::condition::{Condition, Operator};
use crate::jdf::Jdf;
use crate::error::QueryError;

use serde_json::{Value, Map};

use std::sync::Arc;
use std::thread;

#[derive(Debug, PartialEq)]
enum Expression {
    AsteriskInArray(String),
    Nomal(String)
}

impl Expression {
    pub fn from(s: &str) -> Self {
        if s.contains("[*]") {
            Expression::AsteriskInArray(s.to_string())
        } else {
            Expression::Nomal(s.to_string())
        }
    }

    pub fn as_str(&self) -> String {
        match &*self {
            Expression::AsteriskInArray(s) => s.to_string(),
            Expression::Nomal(s) => s.to_string()
        }
    }

    pub fn is_ast(&self) -> bool {
        match *self {
          Expression::AsteriskInArray(_) => true,
          _ => false
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    pub select: Expression,
    pub alias: String,
    pub condition: Condition,
    pub operator: Option<Operator>,
    pub left: Option<Expression>,
    pub right: Option<String>,
}

impl Statement {
    pub fn new(s: &str) -> Result<Self, QueryError> {
        let mut iter = s.split_whitespace();
        let select = iter.next().unwrap_or("-");
        let _as = iter.next();
        let alias = iter.next().unwrap_or(".");
        let condition = iter.next().unwrap_or("-");

        let cond = Condition::from(condition);
        match cond {
            Condition::When => {
                let left = iter.next().unwrap_or("- ");
                let operator = iter.next().unwrap_or("-");
                let right = iter.next().unwrap_or(" -");
                Ok(Statement {
                    select: Expression::from(select),
                    alias: alias.to_string(),
                    condition: cond,
                    operator: Some(Operator::from(operator)),
                    left: Some(Expression::from(left)),
                    right: Some(right.to_string())
                })
            },
            Condition::NoCondition | Condition::Append => {
              Ok(Statement {
                  select: Expression::from(select),
                  alias: alias.to_string(),
                  condition: cond,
                  operator: None,
                  left: None,
                  right: None
              })
            },
            Condition::Unknown => Err(QueryError {})
        }
    }
}

pub struct Query {
    jdf: Jdf,
}

impl Query {
    pub fn new(jdf: Jdf) -> Self {
        Query { jdf: jdf}
    }

    pub fn execute(&self, stmt: Statement) -> Value {
        let mut mp = self.jdf.to_map();
        let mp = Arc::new(self.jdf.to_map());
        Value::Object(execute_(mp, stmt))
    }

    pub fn multi_execute(&self, stmts: Vec<Statement>) -> () {
        let mp = Arc::new(self.jdf.to_map());
        let mut gather = vec![];

        let mut fixed = vec![];

        for stmt in stmts {
            let mp = mp.clone();
            let wait = thread::spawn(move || {
                execute_(mp, stmt)
            });
            gather.push(wait);
        }


        for wait in gather {
           let s = wait.join().unwrap();
           fixed.push(s);
        }

        let sum_mp = fixed.iter().flat_map(|mp_| mp_.clone()).collect::<Map<String, Value>>();
        println!("{:?}", serde_json::to_string(&sum_mp).unwrap());
    }
}

fn execute_(mp: Arc<Map<String, Value>>, stmt: Statement) -> Map<String, Value> {
    let mut ret_mp: Map<String, Value> = Map::new();
    let mp = mp.clone();
    let v = stmt.condition.evaluate(&mp, &stmt);
    ret_mp.insert(stmt.alias, v);

    ret_mp
}
