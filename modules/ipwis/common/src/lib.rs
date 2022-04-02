use anyhow::Result;
use avusen::function::Function;

pub trait Ipwis {
    fn call(&self, func: &Function) -> Result<String>;
}
