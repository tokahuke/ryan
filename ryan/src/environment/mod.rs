/// The Ryan import system.
pub mod loader;
/// Ryan native extensions.
pub mod native;

pub use loader::{DefaultImporter, ImportLoader, NoImport};
pub use native::{NativePatternMatch, BUILT_INS};
use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Debug, rc::Rc};

use self::loader::ImportState;
use crate::{
    parser::{Format, Value},
    rc_world,
};

/// The environment on which a Ryan program operates.
#[derive(Debug, Clone)]
pub struct Environment {
    import_state: Rc<RefCell<ImportState>>,
    /// The name of the current model. It can be `None` if no module is set. This happens
    /// when, e.g., executing Ryan from a supplied string without any extra configuration.
    pub current_module: Option<Rc<str>>,
    built_ins: Rc<HashMap<Rc<str>, Value>>,
}

impl Environment {
    /// Creates a new environment with the default settings (default importer and default
    /// built_ins) with an optional current module name.
    pub fn new(module: Option<&str>) -> Environment {
        let mut builder = Environment::builder();
        if let Some(module) = module {
            builder = builder.module(module);
        }
        builder.build()
    }

    /// Creates an environment builder. Use this to tweak Ryan.
    pub fn builder() -> EnvironmentBuilder {
        EnvironmentBuilder {
            import_loader: Box::new(DefaultImporter),
            current_module: None,
            built_ins: None,
        }
    }

    /// Returns the value associated with a given builtin name.
    pub fn builtin(&self, id: &str) -> Option<Value> {
        self.built_ins.get(id).map(Clone::clone)
    }

    /// Tries to push an import to the import stack.
    fn try_push_import(&self, path: &str) -> Result<Environment, Box<dyn Error + 'static>> {
        let resolved = self
            .import_state
            .borrow_mut()
            .try_push_import(self.current_module.as_deref(), path)?;
        Ok(Environment {
            import_state: self.import_state.clone(),
            current_module: Some(resolved),
            built_ins: self.built_ins.clone(),
        })
    }

    /// Loads a module as a given [`Format`] from a supplied path using the currently
    /// configured loader.
    pub fn load(&self, format: Format, path: &str) -> Result<Value, Box<dyn Error + 'static>> {
        if let Some(value) = self.import_state.borrow().loaded.get(path) {
            return Ok(value.clone());
        }

        let sub_environment = self.try_push_import(path)?;
        let read = self.import_state.borrow().import_loader.load(
            sub_environment
                .current_module
                .as_deref()
                .expect("import stack not empty"),
        )?;
        let value = format.load(sub_environment, read)?;
        self.import_state.borrow_mut().import_stack.pop();

        self.import_state
            .borrow_mut()
            .loaded
            .insert(rc_world::str_to_rc(path), value.clone());

        Ok(value)
    }
}

/// A builder for [`Environment`]s. Use [`Environment::builder`] to create a new builder.
pub struct EnvironmentBuilder {
    import_loader: Box<dyn ImportLoader>,
    current_module: Option<Rc<str>>,
    built_ins: Option<Rc<HashMap<Rc<str>, Value>>>,
}

impl EnvironmentBuilder {
    /// Builds the environment with the supplied configurations.
    pub fn build(self) -> Environment {
        Environment {
            import_state: Rc::new(RefCell::new(ImportState {
                import_loader: self.import_loader,
                loaded: Default::default(),
                import_stack: Default::default(),
            })),
            current_module: self.current_module,
            built_ins: self
                .built_ins
                .unwrap_or_else(|| BUILT_INS.with(Clone::clone)),
        }
    }

    /// Sets the current module name for the environment.
    pub fn module<F>(mut self, module: F) -> Self
    where
        F: AsRef<str>,
    {
        self.current_module = Some(rc_world::str_to_rc(module.as_ref()));
        self
    }

    /// The the import loader for the environment.
    pub fn import_loader<L>(mut self, import_loader: L) -> Self
    where
        L: 'static + ImportLoader,
    {
        self.import_loader = Box::new(import_loader);
        self
    }

    /// Sets the built_ins for the environment.
    pub fn built_ins(mut self, built_ins: Rc<HashMap<Rc<str>, Value>>) -> Self {
        self.built_ins = Some(built_ins);
        self
    }
}
