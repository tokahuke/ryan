#![allow(unused_unsafe)] // Some funky behavior in VSCode...
#![allow(non_snake_case)] // It's JavaScript!

mod utils;

use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use ryan::parser::Value;

fn ryan_to_js(value: &Value) -> Result<JsValue, JsValue> {
    match value {
        Value::Null => Ok(JsValue::NULL),
        Value::Bool(true) => Ok(JsValue::TRUE),
        Value::Bool(false) => Ok(JsValue::FALSE),
        Value::Integer(int) => Ok(JsValue::from_f64(*int as f64)),
        Value::Float(float) => Ok(JsValue::from_f64(*float)),
        Value::Text(text) => Ok(JsValue::from_str(text)),
        Value::List(list) => Ok(JsValue::from(
            list.iter()
                .map(|item| ryan_to_js(item))
                .collect::<Result<Array, _>>()?,
        )),
        Value::Map(dict) => Ok({
            let object = Object::new();

            for (key, value) in dict.iter() {
                let serialized = ryan_to_js(value)?;
                // Unsafety: none whatsoever. Just an annoying editor...
                unsafe {
                    js_sys::Reflect::set(&object, &JsValue::from_str(key), &serialized)?;
                }
            }

            object.into()
        }),
        val => Err(JsError::new(&format!("Unrepresentable value: {val}")).into()),
    }
}

/// This is a patch for a function missing in Ryan as of `0.1.0`.
fn value_from_str(s: &str) -> Result<Value, ryan::Error> {
    let env = ryan::Environment::new(None);
    let parsed = ryan::parser::parse(&s).map_err(ryan::Error::Parse)?;
    let value = ryan::parser::eval(env, &parsed).map_err(ryan::Error::Eval)?;

    Ok(value)
}

/// This is a patch for a function missing in Ryan as of `0.1.0`.
fn value_from_str_with_filename(filename: &str, s: &str) -> Result<Value, ryan::Error> {
    let env = ryan::Environment::new(Some(filename));
    let parsed = ryan::parser::parse(&s).map_err(ryan::Error::Parse)?;
    let value = ryan::parser::eval(env, &parsed).map_err(ryan::Error::Eval)?;

    Ok(value)
}

/// This is a patch for a function missing in Ryan as of `0.1.0`.
pub fn value_from_str_with_env(env: &ryan::Environment, s: &str) -> Result<Value, ryan::Error> {
    let parsed = ryan::parser::parse(&s).map_err(ryan::Error::Parse)?;
    let value = ryan::parser::eval(env.clone(), &parsed).map_err(ryan::Error::Eval)?;

    Ok(value)
}

// /// This is a patch for a function missing in Ryan as of `0.1.0`.
// fn value_from_path(path: &str) -> Result<Value, ryan::Error> {
//     let s = std::fs::read_to_string(path).map_err(ryan::Error::Io)?;
//     value_from_str_with_filename(path, &s)
// }

/// Loads a Ryan file from a supplied string and executes it, building a JavaScript
/// object equivalent to the JSON value resulting from this computation. The
/// `current_module` will be set to `None` while executing in this mode.
#[wasm_bindgen]
pub fn fromStr(s: &str) -> Result<JsValue, JsValue> {
    let value = value_from_str(s.into()).map_err(|err| JsError::new(&err.to_string()))?;
    ryan_to_js(&value)
}

/// Loads a Ryan file from a supplied reader and executes it, building a JavaScript object
/// equivalent to the JSON value resulting from this computation. The `current_module`
/// will be set to `filename` while executing in this mode.
#[wasm_bindgen]
pub fn fromStrWithFilename(filename: &str, s: &str) -> Result<JsValue, JsValue> {
    let value = value_from_str_with_filename(filename, s.into())
        .map_err(|err| JsError::new(&err.to_string()))?;
    ryan_to_js(&value)
}

/// Loads a Ryan file from a supplied string and executes it, finally building an instance
/// of type `T`. from the execution outcome. This function takes an [`Environment`] as a
/// parameter, that lets you have fine-grained control over imports, built-in functions and
/// the `current_module` name.
#[wasm_bindgen]
pub fn fromStrWithEnv(env: &Environment, s: &str) -> Result<JsValue, JsValue> {
    let value =
        value_from_str_with_env(&env.0, s.into()).map_err(|err| JsError::new(&err.to_string()))?;
    ryan_to_js(&value)
}

/// The environment on which a Ryan program operates.
#[wasm_bindgen]
pub struct Environment(ryan::Environment);

#[wasm_bindgen]
impl Environment {
    /// Creates an environment builder. Use this to tweak Ryan.
    #[wasm_bindgen]
    pub fn builder() -> EnvironmentBuilder {
        EnvironmentBuilder(ryan::Environment::builder())
    }

    #[wasm_bindgen(getter)]
    pub fn currentModule(&self) -> Option<String> {
        self.0.current_module.as_deref().map(ToString::to_string)
    }

    #[wasm_bindgen(setter)]
    pub fn set_currentModule(&mut self, newCurrent: Option<String>) {
        self.0.current_module = newCurrent.map(std::rc::Rc::from);
    }
}

/// A builder for `Environment`s. Use `Environment.builder` to create a new builder.
#[wasm_bindgen]
pub struct EnvironmentBuilder(ryan::environment::EnvironmentBuilder);

#[wasm_bindgen]
impl EnvironmentBuilder {
    /// Buils the environment with the supplied configurations.
    #[wasm_bindgen]
    pub fn build(self) -> Environment {
        Environment(self.0.build())
    }

    /// Sets the current module name for the environment.
    #[wasm_bindgen]
    pub fn module(self, module: &str) -> Self {
        Self(self.0.module(module))
    }

    /// The the import loader for the environment.
    #[wasm_bindgen]
    pub fn importLoader(self, loader: JsLoader) -> Self {
        Self(self.0.import_loader(loader))
    }
}

/// This is a loader that is specific to WASM, since WASM is a _much_ more hermetic
/// enviroment than other arhcitectures. In WASM, we cannot be trust on the existence of
/// a filesystem or of environment variables, things that the standard Ryan loader depends
/// on.
///
/// This loader implementation uses the properties of a given JS object to implement a
/// module resolution tree (a tree of nested dictionaries, where the leaves are stringss),
/// a poorman's filesystem of sorts.
///
/// # Note
///
/// Unfortunately, the Rust `Loader` trait is not `async`. Therefore, loading from URLs is
/// not currently suported.
#[derive(Debug)]
#[wasm_bindgen]
pub struct JsLoader {
    modules: JsValue,
}

#[wasm_bindgen]
impl JsLoader {
    #[wasm_bindgen(constructor)]
    pub fn new(modules: JsValue) -> JsLoader {
        JsLoader { modules }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Cannot import beyond virtual system root: {path}")]
struct ImportBeyondRoot {
    path: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Import error from JS: {error}")]
struct ImportError {
    error: String,
}

impl ryan::environment::ImportLoader for JsLoader {
    fn resolve(
        &self,
        current: Option<&str>,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error + 'static>> {
        // Your basic Unix-like filesystem logic... (kinda..)
        let current = current.unwrap_or("");
        let full_path = || current.split('/').chain(path.split('/'));
        let mut stack = vec![];
        for element in full_path() {
            match element {
                "." => {}
                "" => stack.clear(),
                ".." => {
                    if stack.pop().is_none() {
                        return Err(Box::new(ImportBeyondRoot {
                            path: {
                                let mut full = String::new();
                                for el in full_path() {
                                    full.push('/');
                                    full += el;
                                }
                                full
                            },
                        }));
                    }
                }
                el => stack.push(el),
            }
        }

        let mut resolved = String::new();
        for elment in stack {
            resolved.push('/');
            resolved += elment;
        }

        Ok(resolved)
    }

    fn load(
        &self,
        path: &str,
    ) -> Result<Box<dyn std::io::Read>, Box<dyn std::error::Error + 'static>> {
        let mut current = self.modules.clone();

        for element in path.split('/') {
            match element {
                "" => current = self.modules.clone(),
                el =>
                // Unsafety: none whatsoever. Just an annoying editor...
                unsafe {
                    current =
                        js_sys::Reflect::get(&current, &JsValue::from_str(el)).map_err(|err| {
                            ImportError {
                                error: err
                                    .as_string()
                                    .unwrap_or_else(|| "!!NOT UTF-8 ENCODED ERROR!!".to_owned()),
                            }
                        })?;
                },
            }
        }

        Ok(Box::new(std::io::Cursor::new(
            current.as_string().ok_or_else(|| {
                Box::new(ImportError {
                    error: format!("Resolved module cannot be represented in UTF-8"),
                })
            })?,
        )))
    }
}
