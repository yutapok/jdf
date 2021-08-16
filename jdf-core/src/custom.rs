use serde_json::{Value, Map};


#[derive(Clone)]
pub enum Custom {
    Addon(Box<dyn Addon + 'static  + Sync + Send>)
}

pub trait Addon: AddonClone {
    fn pipe(&self, jdf_mp: Map<String, Value>, v: Value) -> Value;
}

pub trait AddonClone {
    fn clone_box(&self) -> Box<dyn Addon + 'static  + Sync + Send>;
}

impl<T> AddonClone for T
  where T: 'static + Addon + Clone + Sync + Send,
{
    fn clone_box(&self) -> Box<dyn Addon + 'static  + Sync + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Addon + 'static  + Sync + Send> {
    fn clone(&self) ->  Box<dyn Addon + 'static  + Sync + Send> {
        self.clone_box()
    }
}


