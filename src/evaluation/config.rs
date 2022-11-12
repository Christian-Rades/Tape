use ext_php_rs::{call_user_func, types::Zval, convert::{IntoZvalDyn, IntoZval}, flags::DataType, ffi::_zval_struct};

use anyhow::{anyhow, Result};

use super::{environment::Filter, value::TaggedValue};

pub struct Config {
    twig_env: Zval,
}

impl Config {
    pub fn new(twig_env: Zval) -> Self {
        Config { twig_env }
    }

    pub fn get_function(&self, name: &str) -> Result<Zval> {
        let funtions = call_user_func!(build_callable(&self.twig_env, "getFunctions"))
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let func = if let Some(Some(f)) = funtions.array().map(|a| a.get(name)) {
            f
        } else {
            return Err(anyhow!("function {} not found", name));
        };

        call_user_func!(build_callable(func, "getCallable")).map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn get_filter(&self, name: &str) -> Result<Filter> {
        let funtions = call_user_func!(build_callable(&self.twig_env, "getFilters"))
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let func = if let Some(Some(f)) = funtions.array().map(|a| a.get(name)) {
            f
        } else {
            return Err(anyhow!("function {} not found", name));
        };

        let callable = call_user_func!(build_callable(func, "getCallable")).map_err(|e| anyhow::anyhow!("{}", e))?;
        let env = ObjAsParamHack{ inner: self.twig_env.shallow_clone() };

        if call_user_func!(build_callable(func, "needsEnvironment")).map_err(|e| anyhow::anyhow!("{}", e))?.bool().unwrap_or_default() {

            Ok(Box::new(move |params: &Vec<TaggedValue>| -> Result<TaggedValue> {
                let mut z_params: Vec<&dyn IntoZvalDyn> = params.iter().map(|p| p as &dyn IntoZvalDyn).collect();
                z_params.insert(0, &env);
                callable.try_call(z_params).map(|zv| TaggedValue::Zval(zv)).map_err(|err| anyhow!("{}", err))
            }))

        } else {

            Ok(Box::new(move |params: &Vec<TaggedValue>| -> Result<TaggedValue> {
                callable.try_call(params.iter().map(|p| p as &dyn IntoZvalDyn).collect()).map(|zv| TaggedValue::Zval(zv)).map_err(|err| anyhow!("{}", err))
            }))

        }
    }
}

fn build_callable(zv: &Zval, fn_name: &str) -> Zval {
    let mut callable = Zval::new();
    callable.set_array(vec![
        zv.shallow_clone(),
        Zval::try_from(fn_name).expect("could not create php string"),
    ]);
    return callable;
}

struct ObjAsParamHack{inner: Zval}

impl Clone for ObjAsParamHack {
    fn clone(&self) -> Self {
        Self { inner: self.inner.shallow_clone() }
    }
}

impl IntoZval for ObjAsParamHack {
    const TYPE: DataType = DataType::Reference;

    fn into_zval(self, _persistend: bool) -> Result<_zval_struct, ext_php_rs::error::Error> {
        Ok(self.inner)
    }

    fn set_zval(self, zv: &mut Zval, _persistent: bool) -> Result<(), ext_php_rs::error::Error> {
        let Self { inner } = self;
        *zv = inner;
        Ok(())
    }
}
