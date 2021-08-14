use crate::statement::{Condition, Operator, Expression, Statement};
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
                self.jdf_mp.get(&stmt.select.as_str()).unwrap_or(&Value::Null).clone()
            },
            Condition::When => self.when(stmt),
            Condition::Append => self.append(stmt),
            Condition::ArrayMap => self.array_map(stmt),
            Condition::FlatMap => self.flat_map(stmt),
            Condition::Unknown => Value::Null
        }
    }


    fn keys_with_ast(&self, stmt_exp: Expression) -> Vec<String> {
        let src = stmt_exp.as_str();
        let dst_v = self.jdf_mp.keys();
        dst_v
          .filter(|dst| self.partial_match(&src, &dst))
          .map(|dst| dst.clone())
          .collect::<Vec<String>>()
    }

    fn partial_match(&self, src: &str, dst: &str) -> bool {
        let mut flag: u8 = 0;
        let src_s = src
          .chars()
          .filter(|c| self.encount(*c, &mut flag))
          .collect::<String>();

        let mut flag: u8 = 0;
        let dst_s = dst
          .chars()
          .filter(|c| self.encount(*c, &mut flag))
          .collect::<String>();

        src_s.eq(&dst_s)
    }

    fn encount(&self, c: char, flag: &mut u8) -> bool {
        if c == "[".chars().next().unwrap() {
            *flag = 1;
        }

        if c == "]".chars().next().unwrap() {
            *flag = 0;
        }

        *flag == 0 as u8
    }

    fn flat_map(&self, stmt: &Statement) -> Value {
        let select = stmt.select.as_str();
        let left  = stmt.left.as_ref().unwrap().as_str();
        let select_vec = self.jdf_mp.get(&select);

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
          .map(|obj| (obj.get(&left), obj.get(&right)))
          .filter(|(l, r)| l.is_some() && r.is_some())
          .map(|(l, r)| {
            let alias = stmt.alias.clone();
            (
              if alias == "*" {
                  l.unwrap().as_str().unwrap().to_string()
              } else {
                  format!("{}_{}", stmt.alias, l.unwrap().as_str().unwrap())
              },
              r.unwrap().clone()
            )
          })
          .collect::<Map<String, Value>>();

        Value::Object(mp)
    }

    fn when(&self, stmt: &Statement) -> Value {
        if stmt.is_ast_exp() {
            let left  = stmt.left.as_ref().unwrap().as_str().clone();
            let dst_v = self.jdf_mp.keys();
            dst_v
              .filter(|dst| self.partial_match(&left, &dst))
              .map(|dst| (dst, self.jdf_mp.get(dst).unwrap_or(&Value::Null)))
              .filter(|(_dst, left)| self.when_case(left, stmt.operator.as_ref().unwrap(), stmt.right.as_ref().unwrap()))
              .map(|(dst, _)| self.mapping_index(stmt.select.as_str(), self.extract_index(dst.to_string())))
              .find_map(|dst| self.jdf_mp.get(&dst))
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

    fn extract_index(&self, src: String) -> Vec<String> {
        let mut new_v = Vec::new();
        let mut flag: u8 = 0;

        let mut str_vec: Vec<char> = Vec::new();
        for c in src.chars(){
            if !self.encount(c, &mut flag){
                str_vec.push(c)
            } else {
                new_v.push(str_vec.iter().filter(|c| **c != "[".chars().next().unwrap()).collect::<String>());
                str_vec = Vec::new();
            }
        }

        new_v.iter().filter(|s| !s.is_empty()).map(|s| s.to_string()).collect::<Vec<String>>()

    }

    fn mapping_index(&self, src: String, indexes: Vec<String>) -> String {
        let mut index_itr = indexes.iter();
        src
          .chars()
          .map(|c|
            if c == "*".chars().next().unwrap() {
              index_itr.next().map(|i| i.to_string()).unwrap_or(c.to_string())
            } else {
              c.to_string()
            }
          )
          .collect::<String>()
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
            let select_v = self.keys_with_ast(stmt.select.clone());
            let v_v = select_v
              .iter()
              .filter_map(|select_s| self.jdf_mp.get(select_s))
              .filter(|v| !v.is_null())
              .map(|v| v.clone())
              .collect::<Vec<Value>>();

            Value::Array(v_v)
        }
    }

    fn array_map(&self, stmt: &Statement) -> Value {
        let v = self.append(stmt);
        let addon_name = stmt.addon.as_ref().unwrap_or(&"-".to_string()).clone();
        let new_v = match &self.extention {
          Some(mp) => match mp.get(&addon_name) {
            Some(Custom::Addon(addon)) => {
              v.as_array()
                .unwrap_or(&Vec::with_capacity(0))
                .iter()
                .enumerate()
                .map(|(ix, v)| addon.pipe(self.jdf_mp.clone(), ix as i64, v.clone()))
                .collect::<Vec<Value>>()
            },
            None => Vec::with_capacity(0)
          },
          None => Vec::with_capacity(0)
        };

        Value::Array(new_v)
    }
}
