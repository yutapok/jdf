extern crate jdf_core;

use jdf_core::jdf::Jdf;
use jdf_core::query::Query;
use jdf_core::statement::Statement;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdout, Write, BufWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut reader = BufReader::new(io::stdin());
    let mut buf = String::new();

    let out = stdout();
    let mut out = BufWriter::new(out.lock());


    let stmts = BufReader::new(File::open(&args[1])?)
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

    Ok(())


}

