pub mod ast;
pub mod parser;
use std::{path::PathBuf, collections::HashMap, fs::File, io::Read, rc::Rc};

use self::ast::Content;
pub use self::{ast::{Module, Template, Extension}, parser::parse};

use anyhow::{anyhow, Result};



pub struct Loader {
    root_dir: PathBuf,
    modules: HashMap<String, Rc<Template>>
}

impl Loader {
    pub fn new(root: PathBuf) -> Self {
        Self { root_dir: root, modules: HashMap::default()}
    }

    pub fn load<T: AsRef<str>>(&mut self, template: T) -> Result<Rc<Template>> {
        match self.modules.get(template.as_ref()) {
            Some(t) => Ok(t.to_owned()),
            None => {
                match self.read_file(template.as_ref())? {
                    Module::Template(mut tpl) => {
                        tpl = self.load_includes(tpl)?;
                        self.modules.insert(template.as_ref().into(), Rc::new(tpl));
                        Ok(self.modules[template.as_ref()].to_owned())
                    }
                    Module::Extension(_) => todo!()
                }
            }
        }
    }

    fn read_file(&mut self, name: &str) -> Result<Module> {
        let fpath = self.root_dir.join(name);
        let  mut file = File::open(fpath)?;

        let mut buf = String::default();
        file.read_to_string(&mut buf)?;

        parse(name.to_string(), &buf)
    }

    fn load_includes(&mut self, template: Template) -> Result<Template> {
        let mut replace_fn = Box::new(|content: Content| -> Content {
            match content {
                Content::Statement(ast::Stmt::Include(name)) => {
                    match self.read_file(&name).expect("todo!!") {
                        Module::Template(tpl) => tpl.into_block(),
                        _ => todo!()
                    }
                },
                _ => content
            }
        });

        Ok(template.replace_includes(&mut replace_fn))
    }

    fn resolve_to_template(&mut self, module: &Extension) -> Result<&Template> {
        todo!()
    }
}
