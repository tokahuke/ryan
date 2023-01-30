/// The Ryan import system.
pub mod loader;
/// Ryan native extensions.
pub mod native;

pub use loader::{DefaultImporter, ImportLoader, NoImport};
pub use native::{NativePatternMatch, BUILTINS};
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
    /// when, e.g., executin Ryan from a supplied string without any extra configuration.
    pub current_module: Option<Rc<str>>,
    builtins: Rc<HashMap<Rc<str>, Value>>,
}

impl Environment {
    /// Creates a new environment with the default settings (default importer and default
    /// builtins) with an optional current module name.
    pub fn new(module: Option<&str>) -> Environment {
        Environment {
            import_state: Rc::default(),
            current_module: module.map(|f| rc_world::str_to_rc(f)),
            builtins: self::native::BUILTINS.with(Clone::clone),
        }
    }

    /// Creates an environment builder. Use this to tweak Ryan.
    pub fn builder() -> EnvironmentBuilder {
        EnvironmentBuilder {
            import_loader: Box::new(DefaultImporter),
            current_module: None,
            builtins: self::native::BUILTINS.with(Clone::clone),
        }
    }

    /// Returs the value associated with a given builtin name.
    pub fn builtin(&self, id: &str) -> Option<Value> {
        self.builtins.get(id).map(Clone::clone)
    }

    /// Tries to push an import to the improt stack.
    fn try_push_import(&self, path: &str) -> Result<Environment, Box<dyn Error + 'static>> {
        let resolved = self
            .import_state
            .borrow_mut()
            .try_push_import(self.current_module.as_deref(), path)?;
        Ok(Environment {
            import_state: self.import_state.clone(),
            current_module: Some(resolved),
            builtins: self.builtins.clone(),
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
    builtins: Rc<HashMap<Rc<str>, Value>>,
}

impl EnvironmentBuilder {
    /// Buils the environment with the supplied configurations.
    pub fn build(self) -> Environment {
        Environment {
            import_state: Rc::new(RefCell::new(ImportState {
                import_loader: self.import_loader,
                loaded: Default::default(),
                import_stack: Default::default(),
            })),
            current_module: self.current_module,
            builtins: self.builtins,
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

    /// Sets the builtins for the environment.
    pub fn builtins(mut self, builtins: Rc<HashMap<Rc<str>, Value>>) -> Self {
        self.builtins = builtins;
        self
    }
}
