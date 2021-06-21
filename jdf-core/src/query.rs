use crate::statement::{Condition, Operator, Expression, Statement};
use crate::jdf::Jdf;
use crate::error::QueryError;

use serde_json::{Value, Map};

use std::sync::Arc;
use std::thread;


pub struct Query {
    inner: Arc<QueryInner>
}

impl Query {
    pub fn new(jdf: Jdf) -> Self {
        Query { inner: Arc::new(QueryInner::new(jdf)) }
    }

    pub fn multi_execute(&mut self, stmts: Vec<Statement>) -> () {
        let mut local_self = self.inner.clone();
        let mut gather = vec![];
        let mut fixed = vec![];

        for stmt in stmts {
            let mut local_self = self.inner.clone();
            let wait = thread::spawn(move || {
                local_self.execute_(stmt)
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

    pub fn execute(&mut self, stmt: Statement) -> Value {
        Value::Object(self.inner.execute_(stmt))
    }

}

struct QueryInner {
    jdf_mp: Map<String, Value>,
}


impl QueryInner {
    pub fn new(mut jdf: Jdf) -> Self {
        jdf.convert();
        QueryInner { jdf_mp: jdf.to_map() }
    }

    fn debug(&self) -> () {
        println!("{:?}", self.jdf_mp);
    }

    fn extract_as_str(&self, s: &str) -> Option<String> {
        match self.jdf_mp.get(s) {
            Some(Value::String(s)) => Some(s.to_string()),
            Some(Value::Number(i)) => Some(i.to_string()),
            _ => None
        }
    }

    fn execute_(&self, stmt: Statement) -> Map<String, Value> {
        let mut ret_mp: Map<String, Value> = Map::new();
        //let v = self.evaluate(&stmt);
        ret_mp.insert(stmt.alias.clone(), self.evaluate(&stmt));
        ret_mp
    }


    fn evaluate(&self, stmt: &Statement) -> Value {
        match stmt.condition {
            Condition::NoCondition =>  {
                self.jdf_mp.get(&stmt.select.as_str()).unwrap_or(&Value::Null).clone()
            },
            Condition::When => self.when(stmt),
    //        Condition::Append => Value::Array(self.append(mp, stmt.select.as_str())
    //          .iter()
    //          .map(|s| Value::String(s.to_string()))
    //          .collect::<Vec<Value>>()
    //        ),
            Condition::Unknown => Value::Null,
            _ => Value::Null
        }
    }

    fn when(&self, stmt: &Statement) -> Value {
        //TODO: refactor
         let select = self.extract_as_str(&stmt.select.as_str());
         match &stmt.select {
             Expression::AsteriskInArray(_) => {
                 (0..self.jdf_mp.iter().len())
                 .map(|ix| {
                   let select = stmt.select.as_str().replace("*", &ix.to_string());
                   let left_s = stmt.left.as_ref().unwrap().as_str().replace("*", &ix.to_string());
                   let left = self.jdf_mp.get(&left_s).unwrap_or(&Value::Null);
                   (select, left)
                 })
                 .filter(|(select, left)| self.when_case(left, &stmt.operator.as_ref().unwrap(), &stmt.right.as_ref().unwrap()))
                 .find_map(|(select, left)| self.jdf_mp.get(&select))
                 .unwrap_or(&Value::Null)
                 .clone()
             },
             Expression::Nomal(s) => {
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
    }

    fn when_case(&self, left: &Value, operator: &Operator, right: &Value) -> bool {
        match (operator, left, right) {
            (Operator::EQ, Value::String(s1), Value::String(s2)) => s1 == s2,
            (Operator::NEQ, Value::String(s1), Value::String(s2)) => s1 != s2,
            (Operator::EQ, Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Operator::NEQ, Value::Number(n1), Value::Number(n2)) => n1 != n2,
            (Operator::EQ, Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Operator::NEQ, Value::Bool(b1), Value::Bool(b2)) => b1 != b2,
            _ => false
        }
    }


    //fn append(&self, mp: &Arc<Map<String, Value>>, select: String) ->  Vec<String> {
    //    let search = Search::new(&select, mp);
    //    if !search.is_asterisk {
    //        let v_ = Vec::with_capacity(1);
    //        return v_
    //    }

    //    search.results().iter()
    //      .filter(|v| **v != Value::Null)
    //      .map(|k| match k {
    //          Value::String(s) => s.to_string(),
    //          _ => "".to_string()
    //      })
    //      .filter(|opt| !opt.is_empty())
    //      .collect::<Vec<String>>()
    //}

    //fn extract_as_str(mp: &Arc<Map<String, Value>>, s: String) ->  Option<String> {
    //    let v = mp.get(&s).unwrap_or(&Value::Null);
    //    match v {
    //        &Value::String(s) => Some(s),
    //        _ => None
    //    }


    //}
}
