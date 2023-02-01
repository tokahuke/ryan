# JavaScript wrapper for the Rust implementation of the Ryan configuration language.

This is a JavaScript wrapper over the WASM Rust implementation of `ryan`.

For basic usage, this module provides two main functions: `fromStr`, which reads
and executes a Ryan program from a string and `fromStrWithEnv`. If you are wondering, no
function is needed for serialization; you can use the standard `JSON.stringify` function
for that (remeber: all JSON is valid Ryan).

Here is a basic usage example:
```javascript
import { fromStr } from ryan;
const value = fromStr(`
  let lights = 4;
  {
    "picard": lights,
    "gulMadred": lights + 1,
  }
`); // { picard: 4, gulMadred: 5 }
```

## Differences from other Ryan implementations

WASM is a _much_ more hermetic enviroment than other arhcitectures. In WASM, we cannot
rely on the existence of a filesystem or of environment variables, things that the
standard Ryan loader depends on. Therefore, a different approach to module loading is
needed.

This implementation uses the properties of a given JS object to implement a
module resolution tree (a tree of nested dictionaries, where the leaves are stringss),
a poorman's filesystem of sorts. An example of how to set up a module system is shown
below:

```javascript
import { Environment, JsLoader, fromStrWithEnv } from "ryan"
const env = Environment
  .builder()
  .loader(new JsLoader({
    "a.ryan": "[1,2,3]",
    "submodule": {
      // Relative import works as expected
      "b.ryan": "import \"../a.ryan\"",
      // As does absolute import:
      "c.ryan": "import \"/a.ryan\"",
    }
  }))
  .build();
const value = fromStrWithEnv(env, `
  let lights = 4;
  {
    "picard": lights,
    "gulMadred": lights + 1,
    imported: import "/submodule/c.ryan",
  }
`); // { picard: 4, gulMadred: 5, imported: [1, 2, 3] }
```

### Note

Unfortunately, the Rust `Loader` trait is not `async`. Therefore, loading from URLs is
not currently suported.


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
* [The Rust docs](https://docs.rs/ryan) also have good info, even if you don't care about Rust.
* [Syntax highlighting for VSCode](https://marketplace.visualstudio.com/items?itemName=PedroBArruda.ryan-syntax-highlighting).

## Limitations of this library

As stated above, WASM works in a more constrained manner than other architectures. This
has led to constraints on how Ryan operates in the browser when it comes to the import
system. See the example above for a detailed explanation.

Also, some of the features present in the Rust implementation are not yet exposed. If you
do have a need for any additional feature to be exposed, please file an issue in the
official Ryan repository.
