# Python wrapper for the Rust implementation of the Ryan configuration language.

For basic usage, this module provides two main functions: `ryan.from_str`, which reads
and executes a Ryan program from a string, and `ryan.from_path`, which reads and
executes a Ryan program from a file. If you are wondering, no function is needed for
serialization; you can use the standard `json` package for that (remeber: all JSON is
valid Ryan).

## How to use Ryan

You can use Ryan in your project via pip:
```bash
pip install ryan-lang
```
Additionally, the Ryan CLI might be useful to have for testing and debugging. See 
[the main page](https://github.com/tokahuke/ryan) for the project for more information.

## Resources for Ryan

* [Main project page](https://github.com/tokahuke/ryan) with more information.
* [The Book of Ryan](https://tokahuke.github.io/book-of-ryan/) _(WIP. New episodes every week!)_.
* [Try out Ryan in your browser](https://tokahuke.github.io/ryan-online/).
* [The Rust docs](https://docs.rs/ryan) also have good info, even if you don't care about Rust.
* [Syntax highlighting for VSCode](https://marketplace.visualstudio.com/items?itemName=PedroBArruda.ryan-syntax-highlighting).

## Limitations of this library

By now, only deserializing is supported. In the future, this wrapper might also get the
full environment API exposed. If you have an usecase for that, please don't hesitate and
open an issue in the Ryan repository.
