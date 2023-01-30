# Ryan: a configuration language for the practical programmer

Ryan is a minimal programming language that produces JSON (and therefore YAML) as
output. It has builtin support for variables, imports and function calls while keeping
things simple. The focus of these added features is to reduce code reuse when
maintaining a sizable codebase of configuration files. It can also be used as an
alternative to creating an overly complex CLI interfaces. Unsure on whether a value
should be stored in a file or in an environment variable? Why not declare a huge
configuration file with everything in it? You leave the users to decide where the
values are coming from, giving them a versatile interface while keeping things simple
on your side. Ryan makes that bridge while keeping the user's code short and
maintanable.

## How to use Ryan

You can run Ryan by installing the CLI:
```bash
cargo install ryan-cli
```
> SOON! Coming to a package manager near you...
If you are developing in Rust, you can easily use Ryan directly in your code by running
```bash
cargo install ryan
```
The basic API is purposefully similar to that of `serde_json` and `serde` is supported by
default.

## Resources for Ryan

* The Book of Ryan _(comming soon)_.
* Syntax highlighting for VSCode _(comming soon)_.
* Run Ryan in your browser (thanks to the magic of WASM) _(comming soon)_.

## Isn't this similar to X?

Yes, Ryan is a product of my frustrations with Dhall and Jsonnet. There is plenty of stuff
I like and hate in both languages. Ryan is an opinionated middle term between the two,
featuring:

* A string equality comparator (yeah, that is a thing).
* Type assertions. Ryan is not statically typed, but you can optionally annotate things
to make sure that there is some type conformity.
* Pattern matching. If you like the `match` statement from Python or Rust or whatever that
thing is in Elixir, you will feel right at home.

## Ryan key principles

It might look at first that adding one more thingamajig to your project might be
overly complicated or even (God forbid!) dangerous. However, Ryan was created with
your main concerns in mind and is _purposefully_ limited in scope. Here is how you
**cannot** code a fully functional Pacman game in Ryan:

1. **(Configurable) hermeticity**: there is no `print` statement or any other kind
side-effect to the language itself. The import system is the only way data can get
into Ryan and even that can be easily disabled. Even if Ryan is not completely
hermetic out-of-the-box, it can be made so in a couple of extra lines.
2. **Turing incompleteness**: this has to do mainly with loops. There is no `while`
statement and you cannot recurse in Ryan. While you can iterate through data, you
can do so only in pre-approved ways. This is done in such a way that every Ryan
program is guaranteed to finish executing (eventually).
3. **Immutability**: everything in Ryan is immutable. Once a value is declared, it
stays that way for the remaining of its existence. Of course, you can _shadow_ a
variable by redeclaring it with anouther value, but that will be a completely new
variable.

Of course, one can reconfigure the import system to read from any arbitrary source of
information and can also create _native extensions_ to throw all these guarantees out
of the window. The possibilitities are infinte. However, these are the sane defaults
that are offered out-of-the-box.
