mod evaluation;
mod loader;
use std::path::PathBuf;

use evaluation::{environment::Env, config::Config};
use ext_php_rs::{prelude::*, types::Zval};

use anyhow::Result;
use loader::Loader;

#[php_function]
pub fn render(base_dir: &str, template: &str, data: &mut Zval, twig_env: &mut Zval) -> Result<String> {
    let conf = Config::new(twig_env.shallow_clone());
    let base_dir = PathBuf::from(base_dir);
    let mut loader = Loader::new(base_dir);
    let tpl = loader.load(template)?;
    evaluation::render(tpl, Env::new(data.shallow_clone(), loader, conf))
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
