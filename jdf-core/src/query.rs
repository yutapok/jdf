use crate::statement::{Condition, Statement};
use crate::custom::Custom;

use crate::jdf::Jdf;

use serde_json::{Value, Map};

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;


pub struct Query {
    inner: Arc<QueryInner>
}

impl Query {
    pub fn new(jdf: Jdf, stmts: Vec<Statement>, extention: Option<HashMap<String, Custom>>) -> Self {
        Query {
          inner: Arc::new(QueryInner::new(jdf, stmts, extention)),
        }
    }

    pub fn execute(&mut self) -> Map<String, Value> {
        let mut gather = vec![];
        let stmts = self.inner.clone().stmts.clone();
        let mut fixed: Map<String, Value> = Map::with_capacity(stmts.len());

        for stmt in stmts {
            let local_self = self.inner.clone();
            let wait = thread::spawn(move || {
                local_self.execute_(&stmt)
            });
            gather.push(wait);
        }

        for wait in gather {
            let mut s = wait.join().unwrap();
            fixed.append(&mut s);
        }

        fixed
    }

}

struct QueryInner {
    jdf_mp: Map<String, Value>,
    stmts: Vec<Statement>,
    extention: Option<HashMap<String, Custom>>}


impl QueryInner {
    pub fn new(mut jdf: Jdf, stmts: Vec<Statement>, extention: Option<HashMap<String, Custom>>) -> Self {
        jdf.convert();
        QueryInner { jdf_mp: jdf.to_map(), stmts: stmts, extention}
    }

    fn execute_(&self, stmt: &Statement) -> Map<String, Value> {
        let mut ret_mp: Map<String, Value> = Map::new();
        if stmt.condition == Condition::FlatMap {
            let mut new_mp = self.evaluate(&stmt)
              .as_object()
              .unwrap_or(&Map::with_capacity(0))
              .clone();

            ret_mp.append(&mut new_mp);

        } else {
            ret_mp.insert(stmt.alias.clone(), self.evaluate(&stmt));

        }
        ret_mp
    }


    fn evaluate(&self, stmt: &Statement) -> Value {
        match stmt.condition {
            Condition::NoCondition =>  {
                self.jdf_mp.get(&stmt.select).unwrap_or(&Value::Null).clone()
            },
            Condition::Map => self.map(stmt),
            Condition::FlatMap => self.flat_map(stmt),
            Condition::Unknown => Value::Null
        }
    }

    fn flat_map(&self, stmt: &Statement) -> Value {
        let select = &stmt.select;
        let left  = stmt.left.as_ref().unwrap().as_str().unwrap();
        let select_vec = self.jdf_mp.get(select);

        let right = match stmt.right.as_ref().unwrap(){
            Value::String(s1) => s1.clone(),
            _ => return Value::Null
        };

        if !select_vec.unwrap_or(&Value::Null).is_array(){
            return Value::Null
        }


        let mp = select_vec.unwrap().as_array().unwrap_or(&Vec::with_capacity(0))
          .iter()
          .filter_map(|v| v.as_object())
          .map(|obj| (obj.get(left), obj.get(&right)))
          .filter(|(l, r)| l.is_some() && r.is_some())
          .map(|(l, r)| {
            let alias = stmt.alias.clone();
            (
              if alias == "*" {
                  l.unwrap().as_str().unwrap().to_string()
              } else {
                  format!("{}.{}", stmt.alias, l.unwrap().as_str().unwrap())
              },
              r.unwrap().clone()
            )
          })
          .collect::<Map<String, Value>>();

        Value::Object(mp)
    }

    fn map(&self, stmt: &Statement) -> Value {
        let select = &stmt.select;
        let v = self.jdf_mp.get(select);

        let addon_name = stmt.addon.as_ref().unwrap_or(&"-".to_string()).clone();
        match &self.extention {
          Some(mp) => match mp.get(&addon_name) {
            Some(Custom::Addon(addon)) => addon.pipe(self.jdf_mp.clone(), v.unwrap_or(&Value::Null).clone()),
            None => Value::Null
          },
          None => Value::Null
        }
    }
}
