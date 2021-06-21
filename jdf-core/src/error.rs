use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct QueryError {}


impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueryError occur")
    }
}

impl Error for QueryError {}
