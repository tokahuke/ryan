use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;

use ::ryan::parser::Value;
use pyo3::types::{PyDict, PyList};

fn ryan_to_python(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(().into_py(py)),
        Value::Bool(b) => Ok(b.into_py(py)),
        Value::Integer(int) => Ok(int.into_py(py)),
        Value::Float(float) => Ok(float.into_py(py)),
        Value::Text(text) => Ok(text.into_py(py)),
        Value::List(list) => Ok(PyList::new(
            py,
            list.iter()
                .map(|v| ryan_to_python(py, v))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .into()),
        Value::Map(dict) => Ok(PyDict::from_sequence(
            py,
            dict.iter()
                .map(|(k, v)| Ok((k.to_object(py), ryan_to_python(py, v)?)))
                .collect::<Result<Vec<_>, PyErr>>()?
                .to_object(py),
        )?
        .into()),
        val => Err(PyValueError::new_err(format!(
            "Unrepresentable value: {val}"
        ))),
    }
}

/// This is a patch for a function missing in Ryan as of `0.1.0`.
pub fn value_from_str(s: &str) -> Result<Value, ::ryan::Error> {
    let env = ::ryan::Environment::new(None);
    let parsed = ::ryan::parser::parse(&s).map_err(::ryan::Error::Parse)?;
    let value = ::ryan::parser::eval(env, &parsed).map_err(::ryan::Error::Eval)?;

    Ok(value)
}

/// This is a patch for a function missing in Ryan as of `0.1.0`.
pub fn value_from_str_with_filename(filename: &str, s: &str) -> Result<Value, ::ryan::Error> {
    let env = ::ryan::Environment::new(Some(filename));
    let parsed = ::ryan::parser::parse(&s).map_err(::ryan::Error::Parse)?;
    let value = ::ryan::parser::eval(env, &parsed).map_err(::ryan::Error::Eval)?;

    Ok(value)
}

/// This is a patch for a function missing in Ryan as of `0.1.0`.
pub fn value_from_path(path: &str) -> Result<Value, ::ryan::Error> {
    let s = std::fs::read_to_string(path).map_err(::ryan::Error::Io)?;
    value_from_str_with_filename(path, &s)
}

/// Python wrapper for the Rust implementation of the Ryan configuration language. For
/// basic usage, this module provides two main functions: `ryan.from_str`, which reads
/// and executes a Ryan program from a string, and `ryan.from_path`, which reads and
/// executes a Ryan program from a file. If you are wondering, no function is needed for
/// serialization; you can use the standard `json` package for that (remeber: all JSON is
/// valid Ryan).
#[pymodule]
pub fn ryan(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    /// Loads a Ryan file from a supplied string and executes it, building a python
    /// object equivalent to the JSON value resulting from this computation. The
    /// `current_module` will be set to `None` while executing in this mode.
    #[pyfn(m)]
    fn from_str(py: Python, s: &str) -> PyResult<PyObject> {
        let value =
            value_from_str(s.into()).map_err(|err| PyException::new_err(err.to_string()))?;
        ryan_to_python(py, &value)
    }

    /// Loads a Ryan file from a supplied reader and executes it, building a python object
    /// equivalent to the JSON value resulting from this computation. The `current_module`
    /// will be set to `filename` while executing in this mode.
    #[pyfn(m)]
    fn from_str_with_filename(py: Python, filename: &str, s: &str) -> PyResult<PyObject> {
        let value = value_from_str_with_filename(filename, s.into())
            .map_err(|err| PyException::new_err(err.to_string()))?;
        ryan_to_python(py, &value)
    }

    /// Loads a Ryan file from disk and executes it, building a python object equivalent
    /// to the JSON value resulting from this computation.
    #[pyfn(m)]
    fn from_path(py: Python, path: &str) -> PyResult<PyObject> {
        let value = value_from_path(path).map_err(|err| PyException::new_err(err.to_string()))?;
        ryan_to_python(py, &value)
    }

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
