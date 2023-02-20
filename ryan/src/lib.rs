// #![warn(missing_docs)] // this can be annoying sometimes.
#![forbid(unsafe_code)]

//! # Ryan: a configuration language for the practical programmer
//!
//! Ryan is a minimal programming language that produces JSON (and therefore YAML) as
//! output. It has builtin support for variables, imports and function calls while keeping
//! things simple. The focus of these added features is to reduce code reuse when
//! maintaining a sizable codebase of configuration files. It can also be used as an
//! alternative to creating an overly complex CLI interfaces. Unsure on whether a value
//! should be stored in a file or in an environment variable? Why not declare a huge
//! configuration file with everything in it? You leave the users to decide where the
//! values are coming from, giving them a versatile interface while keeping things simple
//! on your side. Ryan makes that bridge while keeping the user's code short and
//! maintainable.
//!
//! ## How to use Ryan in your program with this crate
//!
//! If you are not big on the fine details of Ryan or creating your own extensions, you
//! can just use the function [`from_path`], which will give you the final Rust object you
//! want from a file you specify. Thanks to [`serde`] and [`serde_json`], this function
//! can be your one-stop-shop for everything Ryan related.
//!
//! However, if you are looking for ways to customize Ryan, the module [`environment::loader`]
//! has the [`environment::ImportLoader`] trait (along with utilities) to configure the
//! import mechanism however you like. On the other hand, the module [`environment::native`]
//! has the interfaces for native extensions. Finally, everything can be put together
//! in an environment using the [`environment::EnvironmentBuilder`].
//!
//! ## Ryan key principles
//!
//! It might look at first that adding one more thingamajig to your project might be
//! overly complicated or even (God forbid!) dangerous. However, Ryan was created with
//! your main concerns in mind and is _purposefully_ limited in scope. Here is how you
//! **cannot** code a fully functional Pacman game in Ryan:
//!
//! 1. **(Configurable) hermeticity**: there is no `print` statement or any other kind
//! side-effect to the language itself. The import system is the only way data can get
//! into Ryan and even that can be easily disabled. Even if Ryan is not completely
//! hermetic out-of-the-box, it can be made so in a couple of extra lines.
//! 2. **Turing incompleteness**: this has to do mainly with loops. There is no `while`
//! statement and you cannot recurse in Ryan. While you can iterate through data, you
//! can do so only in pre-approved ways. This is done in such a way that every Ryan
//! program is guaranteed to finish executing (eventually).
//! 3. **Immutability**: everything in Ryan is immutable. Once a value is declared, it
//! stays that way for the remaining of its existence. Of course, you can _shadow_ a
//! variable by re-declaring it with another value, but that will be a completely new
//! variable.
//!
//! Of course, one can reconfigure the import system to read from any arbitrary source of
//! information and can also create _native extensions_ to throw all these guarantees out
//! of the window. The possibilities are infinite. However, these are the sane defaults
//! that are offered out-of-the-box.
//!
//! # A primer on Ryan
//!
//! In the first place, Ryan, just like YAML, is a superset of JSON. Therefore, every
//! valid JSON is also valid Ryan:
//! ```ryan
//! {
//!     "this": "works",
//!     "that": ["is", "a", "list"],
//!     "how_many_lights": 4,
//!     "billion_dollar_mistake": null
//! }
//! ```
//! However, JSON lacks many of the amenities we have grown so accustomed to:
//! ```ryan
//! // Comments...
//! {
//!     // Omitting annoying quotes:
//!     this: "works",
//!     // Forgiving commas:
//!     that: ["is", "a", "list",],
//!     // Basic maths
//!     how_many_lights: 5 - 1,
//!     billion_dollar_mistake: null,
//! }
//! ```
//! Besides, since we are all about code reusability, defining variables is supported:
//! ```ryan
//! let lights = 4;
//! {
//!     "picard": lights,
//!     "gul_madred": lights + 1,
//! }
//! ```
//! But that is not all! Ryan is a _pattern matching_ language. Everything can be
//! destructured down to its most basic components:
//! ```ryan
//! let { legends: { tanagra, temba, shaka }, .. } = {
//!     participants: ["Picard", "Dathon"],
//!     legends: {
//!         tanagra: "Darmok and Jalad",
//!         temba: "his arms wide",
//!         shaka: "when the walls fell",
//!     },
//! };
//!
//! "Temba, " + temba    // "Temba, his arms wide"
//! ```
//! And last, but not least, you can import stuff, in a variety of ways:
//! ```ryan
//! // Will evaluate the file and import the final value into the variable:
//! let q_episode_list = import "qEpisodes.ryan";
//!
//! // Will import "captain's log" as text, verbatim:
//! let captains_log = import "captainsLog.txt" as text;
//!
//! // Will import value as text from an environment variable:
//! let ensign_name = import "env:ENSIGN" as text;
//!
//! // Will import value as text or provide a default if not set:
//! let cadet_name = import "env:CADET" as text or "Wesley Crusher";
//!
//! // No! No funny imports. Import string must be constant:
//! let a_letter = "Q";
//! let alien_entity = import "env:" + a_letter;    // <= parse error!
//! ```
//! Of course, there is some more to Ryan that this quick tour, but you already get the
//! idea of the key components. To get the full picture, please refer to the book
//! (under construction).
//!

/// Deserializes a Ryan value into a Rust struct using `serde`'s data model.
mod de;
/// The interface between Ryan and the rest of the world. Contains the import system and
/// the native extension system.
pub mod environment;
/// The Ryan language _per se_, with parsing and evaluating functions and the types
/// building the Abstract Syntax Tree.
pub mod parser;
/// The way Ryan allocates strings in memory.
mod rc_world;
/// Utilities for this crate.
mod utils;

pub use crate::de::DecodeError;
pub use crate::environment::Environment;

use serde::Deserialize;
use std::{io::Read, path::Path};
use thiserror::Error;

use crate::parser::{EvalError, ParseError};

/// The errors that may happen while processing Ryan programs.
#[derive(Debug, Error)]
pub enum Error {
    /// An IO error happened (e.g., the file does not exist).
    #[error("Io error: {0}")]
    Io(std::io::Error),
    /// A parse error happened.
    #[error("{0}")]
    Parse(ParseError),
    /// A runtime error happened (e.g, there was a variable missing somewhere).
    #[error("{0}")]
    Eval(EvalError),
    /// An error happened when transforming the final result to JSON.
    #[error("Decode error: {0}")]
    DecodeError(DecodeError),
}

/// Loads a Ryan file from disk and executes it, finally building an instance of type `T`
/// from the execution outcome.
pub fn from_path<P: AsRef<Path>, T>(path: P) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let file = std::fs::File::open(path.as_ref()).map_err(Error::Io)?;
    let decoded = from_reader_with_filename(&path.as_ref().display().to_string(), file)?;

    Ok(decoded)
}

/// Loads a Ryan file from disk and executes it, finally building an instance of type `T`
/// from the execution outcome. This function takes an [`Environment`] as a parameter,
/// that lets you have fine-grained control over imports and built-in functions.
pub fn from_path_with_env<P: AsRef<Path>, T>(env: &Environment, path: P) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let mut patched_env = env.clone();
    patched_env.current_module = Some(path.as_ref().display().to_string().into());
    let file = std::fs::File::open(path.as_ref()).map_err(Error::Io)?;
    let decoded = from_reader_with_env(&patched_env, file)?;

    Ok(decoded)
}

/// Loads a Ryan file from a supplied reader and executes it, finally building an instance
/// of type `T` from the execution outcome. The `current_module` will be set to `None`
/// while executing in this mode.
pub fn from_reader<R: Read, T>(mut reader: R) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let mut string = String::new();
    reader.read_to_string(&mut string).map_err(Error::Io)?;
    let decoded = from_str(&string)?;

    Ok(decoded)
}

/// Loads a Ryan file from a supplied reader and executes it, finally building an instance
/// of type `T` from the execution outcome. The `current_module` will be set to `name`
/// while executing in this mode.
pub fn from_reader_with_filename<R: Read, T>(name: &str, mut reader: R) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let mut string = String::new();
    reader.read_to_string(&mut string).map_err(Error::Io)?;
    let decoded = from_str_with_filename(name, &string)?;

    Ok(decoded)
}

/// Loads a Ryan file from a supplied reader and executes it, finally building an instance
/// of type `T`. from the execution outcome. This function takes an [`Environment`] as a
/// parameter, that lets you have fine-grained control over imports, built-in functions and
/// the `current_module` name.
pub fn from_reader_with_env<R: Read, T>(env: &Environment, mut reader: R) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let mut string = String::new();
    reader.read_to_string(&mut string).map_err(Error::Io)?;
    let decoded = from_str_with_env(env, &string)?;

    Ok(decoded)
}

/// Loads a Ryan file from a supplied string and executes it, finally building an instance
/// of type `T` from the execution outcome. The `current_module` will be set to `None`
/// while executing in this mode.
pub fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let env = Environment::new(None);
    let parsed = parser::parse(&s).map_err(Error::Parse)?;
    let value = parser::eval(env, &parsed).map_err(Error::Eval)?;
    let decoded = value.decode::<T>().map_err(Error::DecodeError)?;

    Ok(decoded)
}

/// Loads a Ryan file from a supplied string and executes it, finally building an instance
/// of type `T` from the execution outcome. The `current_module` will be set to `name`
/// while executing in this mode.
pub fn from_str_with_filename<T>(name: &str, s: &str) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let env = Environment::new(Some(name));
    let parsed = parser::parse(&s).map_err(Error::Parse)?;
    let value = parser::eval(env, &parsed).map_err(Error::Eval)?;
    let decoded = value.decode().map_err(Error::DecodeError)?;

    Ok(decoded)
}

/// Loads a Ryan file from a supplied string and executes it, finally building an instance
/// of type `T`. from the execution outcome. This function takes an [`Environment`] as a
/// parameter, that lets you have fine-grained control over imports, built-in functions and
/// the `current_module` name.
pub fn from_str_with_env<T>(env: &Environment, s: &str) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let parsed = parser::parse(&s).map_err(Error::Parse)?;
    let value = parser::eval(env.clone(), &parsed).map_err(Error::Eval)?;
    let decoded = value.decode().map_err(Error::DecodeError)?;

    Ok(decoded)
}
