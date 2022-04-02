use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct Function {
	pub method: String,
	pub msg: String,
}

pub trait Ipwis {
	fn call(&self, func: &Function) -> Result<String, String>;
}
