extern crate jdf_core;

use jdf_core::jdf::Jdf;
use jdf_core::query::Query;
use jdf_core::statement::Statement;
use jdf_core::error;
use jdf_core::error::JdfError;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdout, Write, BufWriter};


const HELP: &'static str = r#"
USAGE:
       jdf [FLAGS]
FLAGS:
       <NONE>               Show flatten json
       -c <Path/to/JQL>     Use Query
       -h                   Prints help information
"#;

#[derive(PartialEq)]
enum ExecType {
    Pure,
    Query,
    Help
}


impl ExecType {
    pub fn from(s: Vec<String>) -> Self {
        if s.len() < 2 {
            return ExecType::Pure
        }
        let sub = &s[1];
        match sub.as_str() {
          "-c" => ExecType::Query,
          "-h" => ExecType::Help,
          _ => ExecType::Pure
        }
    }
}

fn main() -> Result<(), JdfError> {
    let args = env::args().collect::<Vec<String>>();

    let mut reader = BufReader::new(io::stdin());
    let mut buf = String::new();

    let out = stdout();
    let mut out = BufWriter::new(out.lock());

    let e_type = ExecType::from(args.clone());

    match e_type {
        ExecType::Query => {
            if args.len() < 3 {
                return Err(error!("Usage: -c  /path/to/jql_file>"))
            }
            let stmts = BufReader::new(File::open(&args[2])?)
              .lines()
              .filter_map(|l| l.ok())
              .filter_map(|l| Statement::new(&l).ok())
              .collect::<Vec<Statement>>();
            while reader.read_line(&mut buf)? > 0 {
                let jdf = Jdf::new(buf.to_string());
                let mut q = Query::new(jdf, stmts.clone());
                let new_mp = q.execute();

                writeln!(out, "{}", serde_json::to_string(&new_mp).unwrap()).unwrap();

                buf.clear();

            }
        },
        ExecType::Pure => {
            while reader.read_line(&mut buf)? > 0 {
                let mut jdf = Jdf::new(buf.to_string());
                jdf.convert();

                writeln!(out, "{}", serde_json::to_string(&jdf.to_map()).unwrap()).unwrap();
                buf.clear();
            }
        },
        ExecType::Help => writeln!(out, "{}", HELP).unwrap()

    }
    Ok(())


}

