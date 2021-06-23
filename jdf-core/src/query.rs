use crate::statement::{Condition, Operator, Statement};
use crate::jdf::Jdf;

use serde_json::{Value, Map};

use std::sync::Arc;
use std::thread;


pub struct Query {
    inner: Arc<QueryInner>
}

impl Query {
    pub fn new(jdf: Jdf, stmts: Vec<Statement>) -> Self {
        Query {
          inner: Arc::new(QueryInner::new(jdf, stmts)),
        }
    }

    pub fn execute(&mut self) -> Map<String, Value> {
        let mut gather = vec![];
        let mut fixed = vec![];
        let stmts = self.inner.clone().stmts.clone();
        for stmt in stmts {
            let local_self = self.inner.clone();
            let wait = thread::spawn(move || {
                local_self.execute_(&stmt)
            });
            gather.push(wait);
        }


        for wait in gather {
           let s = wait.join().unwrap();
           fixed.push(s);
        }

        fixed
          .iter()
          .flat_map(|mp_| mp_.clone())
          .collect::<Map<String, Value>>()
    }

}

struct QueryInner {
    jdf_mp: Map<String, Value>,
    stmts: Vec<Statement>
}


impl QueryInner {
    pub fn new(mut jdf: Jdf, stmts: Vec<Statement>) -> Self {
        jdf.convert();
        QueryInner { jdf_mp: jdf.to_map(), stmts: stmts}
    }

    fn execute_(&self, stmt: &Statement) -> Map<String, Value> {
        let mut ret_mp: Map<String, Value> = Map::new();
        ret_mp.insert(stmt.alias.clone(), self.evaluate(&stmt));
        ret_mp
    }


    fn evaluate(&self, stmt: &Statement) -> Value {
        match stmt.condition {
            Condition::NoCondition =>  {
                self.jdf_mp.get(&stmt.select.as_str()).unwrap_or(&Value::Null).clone()
            },
            Condition::When => self.when(stmt),
            Condition::Append => self.append(stmt),
            Condition::Unknown => Value::Null
        }
    }

    fn when(&self, stmt: &Statement) -> Value {
        let length: usize = self.jdf_mp.iter().len();
        if stmt.is_ast_exp() {
            let select_v = stmt.select.as_ast().unwrap().parse_as_vec(length);
            let left_v = stmt.left.as_ref().unwrap().as_ast().unwrap().parse_as_vec(length);

            select_v.iter().zip(left_v.iter()).map(|(select_s, left_s)| {
                (
                  select_s,
                  self.jdf_mp.get(left_s).unwrap_or(&Value::Null)
                )
            })
            .filter(|(_select, left)| self.when_case(left, &stmt.operator.as_ref().unwrap(), &stmt.right.as_ref().unwrap()))
            .find_map(|(select, _left)| self.jdf_mp.get(select))
            .unwrap_or(&Value::Null)
            .clone()
        } else {
            if self.when_case(
                self.jdf_mp.get(&stmt.left.as_ref().unwrap().as_str()).unwrap_or(&Value::Null),
                stmt.operator.as_ref().unwrap(),
                stmt.right.as_ref().unwrap()
            ) {
                self.jdf_mp.get(&stmt.select.as_str()).unwrap_or(&Value::Null).clone()
            } else {
                Value::Null
            }
        }
    }


    fn when_case(&self, left: &Value, operator: &Operator, right: &Value) -> bool {
        match (left, right) {
            (Value::String(s1), Value::String(s2)) => if *operator == Operator::EQ { s1 == s2 } else { s1 != s2 },
            (Value::Number(n1), Value::Number(n2)) => if *operator == Operator::EQ { n1 == n2 } else { n1 != n2 },
            (Value::Bool(b1), Value::Bool(b2)) => if *operator == Operator::EQ { b1 == b2 } else { b1 != b2 },
            _ => false
        }
    }


    fn append(&self, stmt: &Statement) -> Value {
        if !stmt.select.is_ast() {
            Value::Array(Vec::with_capacity(1))
        } else {
            let select_v = stmt.select.as_ast().unwrap().parse_as_vec(self.jdf_mp.iter().len());
            let v_v = select_v
              .iter()
              .filter_map(|select_s| self.jdf_mp.get(select_s))
              .filter(|v| !v.is_null())
              .map(|v| v.clone())
              .inspect(|v| println!("{}", v))
              .collect::<Vec<Value>>();

            Value::Array(v_v)
        }
    }
}
