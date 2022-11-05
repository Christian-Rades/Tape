use ext_php_rs::{types::Zval, call_user_func};

use anyhow::{anyhow, Result};

pub struct Config {
    twig_env: Zval,
}

impl Config {
    pub fn new(twig_env: Zval) -> Self {
        Config { twig_env }
    }

    pub fn get_function(&self, name: &str) -> Result<Zval> {
        let funtions = call_user_func!(build_callable(&self.twig_env, "getFunctions")).map_err(|e| anyhow::anyhow!("{}", e))?;
        let func = if let Some(Some(f)) = funtions.array().map(|a| a.get(name)) {
            f
        } else {
            return Err(anyhow!("function {} not found", name));
        };

        call_user_func!(build_callable(func, "getCallable")).map_err(|e| anyhow::anyhow!("{}", e))
    }

}

fn build_callable(zv: &Zval, fn_name: &str) -> Zval {
    let mut callable = Zval::new();
    callable.set_array(vec![zv.shallow_clone(), Zval::try_from(fn_name).expect("could not create php string")]);
    return callable
}
