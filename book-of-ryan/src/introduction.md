# Introduction

## What is book about?

This book is about getting you started with Ryan, from zero to hero! First of all, before we even start with the language itself, we will show how you can get Ryan to work on your machine. There is a growing list of options available. Then, we will introduce the language syntax, with its most common features. Ryan is a simple and straighforward programming language; you can master it in no time! Lastly, we will show how you can integrate Ryan with your current applications. Probably, you just need to read one or two chapters of this final section, unless you regularly program for a plethora of different languages.

This book is _not_ about implementation details or how to tweak the import system. These more advanced topics deserve a different approach, which is not suitable for people just starting out. If you curious about that, the [API documentation](https://docs.rs/ryan) is a great place to start, even if Rust is not quite your thing.

## Who the hell is Ryan?

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


So, without further ado, shall we begin?
