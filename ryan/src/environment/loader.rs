use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Debug},
    io::{Cursor, Read},
    path::PathBuf,
    rc::Rc,
};
use thiserror::Error;

use crate::{parser::Value, rc_world};

/// The loader trait for Ryan.
pub trait ImportLoader: fmt::Debug {
    /// Returns the _absolute_ import path for a module, given a base and an optional
    /// path. The base might be set to `None` in some cases (e.g., when loading Ryan
    /// from a string, not a file). Each loader implementation can choose how to treat
    /// this value however it's expedient.
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn Error + 'static>>;
    /// Resolves an _absolute_ path into a reader, where a Ryan module can be read from.
    fn load(&self, path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>>;

    /// Overrides a single path to be represented by a different model than would be
    /// represented by this loader.
    fn r#override(self, path: String, value: String) -> Override<Self>
    where
        Self: Sized,
    {
        let mut overrides = HashMap::new();
        overrides.insert(path, Some(value));
        Override {
            loader: self,
            overrides,
        }
    }

    /// Blocks a path from being resolved as a module.
    fn r#block(self, path: String) -> Override<Self>
    where
        Self: Sized,
    {
        let mut overrides = HashMap::new();
        overrides.insert(path, None);
        Override {
            loader: self,
            overrides,
        }
    }

    /// Overrides the value imported by multiple paths. Pass `None` as the value associated
    /// to a key in the hashmap to deny access to a given path.
    fn r#override_many(self, overrides: HashMap<String, Option<String>>) -> Override<Self>
    where
        Self: Sized,
    {
        Override {
            loader: self,
            overrides,
        }
    }

    /// Determines whether a path should be blocked or loaded based on a supplied closure.
    fn filter<F>(self, filter: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> bool,
    {
        Filter {
            loader: self,
            filter,
        }
    }

    /// Change the `resolve` method of the underlying loader.
    fn with_resolver<F, E>(self, resolver: F) -> WithResolver<Self, F>
    where
        Self: Sized,
        F: Fn(Option<&str>, &str) -> Result<String, E>,
    {
        WithResolver {
            loader: self,
            resolver,
        }
    }

    /// Change the `load` method of the underlying loader.
    fn with_loader<F, R, E>(self, loader: F) -> WithLoader<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> Result<R, E>,
        R: 'static + Read,
        E: 'static + Error,
    {
        WithLoader {
            loader: self,
            func: loader,
        }
    }
}

/// The error returned by the [`NoImport`] loader for all modules.
#[derive(Error, Debug)]
#[error("Imports are disabled")]
pub struct NoImportError;

/// An importer that blocks all imports. Use this to make Ryan completely separated from
/// the outside world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoImport;

impl ImportLoader for NoImport {
    fn resolve(
        &self,
        _current: Option<&str>,
        _path: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        Err(Box::new(NoImportError))
    }

    fn load(&self, _path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>> {
        Err(Box::new(NoImportError))
    }
}

/// The default importer for Ryan. This importer will read any file in the system, plus
/// all environment variables, when the module starts with the `env:` prefix. There is
/// the one added restriction that `env:` modules don't have access to load regular files.
/// This happens because the working directory for an environment variable is
/// ill-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefaultImporter;

impl ImportLoader for DefaultImporter {
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        if path.starts_with("env:") {
            Ok(path.to_owned())
        } else {
            let resolved = if let Some(current) = current {
                if current.starts_with("env:") {
                    return Err(Box::new(ImportError::CannotAccessFileSystemFromEnv));
                } else {
                    let mut resolved = PathBuf::from(current);
                    resolved.pop();
                    resolved.push(path);
                    resolved
                }
            } else {
                let mut resolved = std::env::current_dir()?;
                resolved.push(path);
                resolved
            };

            Ok(resolved.to_string_lossy().into_owned())
        }
    }

    fn load(&self, path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>> {
        if path.starts_with("env:") {
            let var = &path["env:".len()..];
            Ok(Box::new(std::io::Cursor::new(std::env::var(var)?)))
        } else {
            Ok(Box::new(std::fs::File::open(path)?))
        }
    }
}

/// Errors that can happen while importing a module.
#[derive(Error, Debug)]
pub enum ImportError {
    /// A module tried to, directly or indirectly, import itself.
    #[error("Circular import detected at {0:?}")]
    CircularImportDetected(Rc<str>),
    /// An environment variable module tried to access the filesystem.
    #[error("Cannot access the filesystem from the environment variable")]
    CannotAccessFileSystemFromEnv,
    /// There is an override for this module and it cannot be accessed.
    #[error("Cannot access the filesystem from the environment variable")]
    ImportPathIsOverridden(Rc<str>),
}

/// The internal state of the import system.
#[derive(Debug)]
pub(super) struct ImportState {
    pub(super) import_loader: Box<dyn ImportLoader>,
    pub(super) loaded: HashMap<Rc<str>, Value>,
    pub(super) import_stack: Vec<Rc<str>>,
}

impl Default for ImportState {
    fn default() -> Self {
        ImportState {
            import_loader: Box::new(DefaultImporter),
            loaded: HashMap::default(),
            import_stack: vec![],
        }
    }
}

impl ImportState {
    pub(super) fn try_push_import(
        &mut self,
        current: Option<&str>,
        path: &str,
    ) -> Result<Rc<str>, Box<dyn Error + 'static>> {
        let path = self.import_loader.resolve(current, path)?;
        let resolved = rc_world::string_to_rc(path);

        if self.import_stack.iter().any(|p| p == &resolved) {
            return Err(Box::new(ImportError::CircularImportDetected(resolved)));
        }

        self.import_stack.push(resolved.clone());

        Ok(resolved)
    }
}

/// The resulting loader for the [`ImportLoader::override`], [`ImportLoader::block`] and
/// [`ImportLoader::override_many`] methods.
#[derive(Debug)]
pub struct Override<L> {
    loader: L,
    overrides: HashMap<String, Option<String>>,
}

impl<L: ImportLoader> ImportLoader for Override<L> {
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        if self.overrides.contains_key(path) {
            Ok(path.to_string())
        } else {
            self.loader.resolve(current, path)
        }
    }

    fn load(&self, path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>> {
        match self.overrides.get(path) {
            Some(Some(overridden)) => Ok(Box::new(Cursor::new(overridden.clone()))),
            Some(None) => Err(Box::new(ImportError::ImportPathIsOverridden(
                rc_world::str_to_rc(path),
            ))),
            None => self.load(path),
        }
    }
}

/// The resulting loader for the [`ImportLoader::filter`] method.
pub struct Filter<L, F> {
    loader: L,
    filter: F,
}

impl<L: Debug, F> Debug for Filter<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Filter {{ loader: {:?} }}", self.loader)
    }
}

impl<L: ImportLoader, F> ImportLoader for Filter<L, F>
where
    F: Fn(&str) -> bool,
{
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        self.loader.resolve(current, path)
    }

    fn load(&self, path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>> {
        if (self.filter)(path) {
            self.load(path)
        } else {
            return Err(Box::new(ImportError::ImportPathIsOverridden(
                rc_world::str_to_rc(path),
            )));
        }
    }
}

/// The resulting loader for the [`ImportLoader::with_resolver`] method.
pub struct WithResolver<L, F> {
    loader: L,
    resolver: F,
}

impl<L: Debug, F> Debug for WithResolver<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WithResolver {{ loader: {:?} }}", self.loader)
    }
}

impl<L: ImportLoader, F, E> ImportLoader for WithResolver<L, F>
where
    F: Fn(Option<&str>, &str) -> Result<String, E>,
    E: 'static + Error,
{
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        (self.resolver)(current, path).map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn load(&self, path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>> {
        self.loader.load(path)
    }
}

/// The resulting loader for the [`ImportLoader::with_loader`] method.
pub struct WithLoader<L, F> {
    loader: L,
    func: F,
}

impl<L: Debug, F> Debug for WithLoader<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WithLoader {{ loader: {:?} }}", self.loader)
    }
}

impl<L: ImportLoader, F, R, E> ImportLoader for WithLoader<L, F>
where
    F: Fn(&str) -> Result<R, E>,
    R: 'static + Read,
    E: 'static + Error,
{
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        self.loader.resolve(current, path)
    }

    fn load(&self, path: &str) -> Result<Box<dyn Read>, Box<dyn Error + 'static>> {
        (self.func)(path)
            .map(|read| Box::new(read) as Box<dyn Read>)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}
