pub mod datetime_parser;

use jdf_core::custom::{Custom, Addon};
use std::collections::HashMap;

use crate::sample::Sample;


pub struct Addons {}


impl Addons {
    pub fn load() -> HashMap<String, Custom> {
        let mut mp = HashMap::new();

        // Add extention
        mp.insert("DATE_EMMBLING".to_string(), Custom::Addon(Box::new(Sample {})));

        //

        mp
    }
}

