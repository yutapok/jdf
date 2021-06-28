extern crate libc;

use std::ffi::{CStr,CString};

use jdf_core::statement::Statement;
use jdf_core::jdf::Jdf;
use jdf_core::query::Query;


#[no_mangle]
pub extern "C" fn flatten(json_s: *const i8) -> *const i8 {
    let json_s = unsafe {
      CStr::from_ptr(json_s).to_string_lossy().into_owned()
    };

    let mut jdf = Jdf::new(json_s);
    jdf.convert();

    let new_json_s = format!("{}", serde_json::to_string(&jdf.to_map()).unwrap());

    CString::new(new_json_s).unwrap().into_raw()

}


#[no_mangle]
pub extern "C" fn query(json_s: *const i8, statements: *const i8) -> *const i8 {
    let json_s = unsafe {
      CStr::from_ptr(json_s).to_string_lossy().into_owned()
    };

    let jdf = Jdf::new(json_s);

    let stmts_s = unsafe {
      CStr::from_ptr(statements)
        .to_string_lossy()
        .into_owned()
    };

    let stmts = stmts_s.clone()
     .split("")
     .into_iter()
     .map(|ss| ss.to_string())
     .filter_map(|v| Statement::new(&v).ok())
     .collect::<Vec<Statement>>();


    let mut q = Query::new(jdf, stmts.clone());

    let ret_mp = q.execute();

    let new_json_s = format!("{}", serde_json::to_string(&ret_mp).unwrap());

    CString::new(new_json_s).unwrap().into_raw()
}
