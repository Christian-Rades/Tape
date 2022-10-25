use std::collections::HashMap;

use ext_php_rs::{convert::FromZval, types::Zval};

use crate::loader::{ast::Setter, Loader, Module};

use anyhow::{anyhow, Result};

use super::{expressions::Evaluate, value::TaggedValue};

pub struct Env {
    globals: Zval,
    stack: Vec<Scope>,
    loader: Loader,
}

type Scope = HashMap<String, TaggedValue>;

impl Env {
    pub fn new(globals: Zval, loader: Loader) -> Self {
        Self {
            globals,
            stack: vec![Scope::default()],
            loader,
        }
    }

    pub fn load_file<T: AsRef<str>>(&mut self, file: T) -> Result<Module> {
        self.loader.load(file)
    }

    pub fn enter_new_scope(mut self) -> Self {
        self.stack.push(Scope::default());
        self
    }
    pub fn exit_scope(mut self) -> Self {
        self.stack.pop();
        self
    }

    pub fn set(&mut self, name: &str, val: TaggedValue) {
        let scope = self.get_scope(name);
        scope.insert(name.to_string(), val);
    }

    pub fn apply_setter(&mut self, setter: &Setter) {
        self.set(
            &setter.target,
            setter.value.eval(self).expect("fix error case"),
        )
    }

    pub fn get(&self, accessor: &str) -> Result<TaggedValue> {
        if accessor.is_empty() {
            return Err(anyhow!("empty varname"));
        }

        if let Some(val) = self.get_from_scope(accessor) {
            return Ok(val);
        }

        match Self::get_rec(&self.globals, accessor) {
            Some(zv) => Ok(TaggedValue::Zval(zv.shallow_clone())),
            None => Err(anyhow!("variable {} was not found", accessor)),
        }
    }

    fn get_from_scope(&self, accessor: &str) -> Option<TaggedValue> {
        let (key, rest) = if accessor.contains('.') {
            accessor.split_once('.').unwrap()
        } else {
            (accessor, "")
        };

        for scope in self.stack.iter().rev() {
            if let Some(val) = scope.get(key) {
                return match val {
                    TaggedValue::Zval(zv) => {
                        Self::get_rec(&zv, rest).and_then(TaggedValue::from_zval)
                    }
                    _ => Some(val.clone()),
                };
            }
        }
        None
    }

    fn get_scope<'env>(&'env mut self, accessor: &'_ str) -> &'env mut Scope {
        let key = accessor.split_once('.').map(|(k, _)| k).unwrap_or(accessor);

        let mut idx = self.stack.len() - 1;
        for (i, scope) in self.stack.iter().enumerate().rev() {
            if scope.contains_key(key) {
                idx = i;
                break;
            }
        }
        self.stack
            .get_mut(idx)
            .expect("env should always contain 1 scope")
    }

    fn get_rec<'a>(val: &'a Zval, accessor: &'_ str) -> Option<&'a Zval> {
        if accessor.is_empty() {
            return Some(val);
        }
        let (key, rest) = if accessor.contains('.') {
            accessor.split_once('.').unwrap()
        } else {
            (accessor, "")
        };

        if val.is_array() {
            let array = val.array()?;
            return Self::get_rec(array.get(key)?, rest);
        }

        if val.is_object() {
            let obj = val.object()?;
            return Self::get_rec(obj.get_property(key).ok()?, rest);
        }
        None
    }
}
