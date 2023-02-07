# The Ryan CLI

The simplest way of interfacing with Ryan is through the Ryan CLI. The CLI is an app that executes your Ryan programs and spits out the final computed JSON either to a file or to the terminal. Since the CLI is a program that you can download in most computers, you can use it independently of your programming environment. In fact, if your language does not support native integration, don't fret: the CLI is purpose built for integration with other programs and applications. 

## Simple usage

Evaluating a Ryan program is as simple as writing:
```sh
ryan my_program.ryan
```
This will execute a file `my_program.ryan`, stored in the current working directory, and print the returned JSON on the screen, with indentations, pretty colors and other niceties. Else, it will print an error message on the screen, saying, more or less cryptically, what is wrong with your code. In case the code executes to valid JSON, the _return status_ will be `0`, else, if there is any problem, the _return status_ will be non-zero (the actual value depends on the error). This is in line with all well behaved programs and your computer will have no problem understanding it.

If you wish to save the calculated value to a file, just use the `>` operator:
```sh
ryan my_program.ryan > output.json
```

In the same vein, you can set environment variables as usual, which (for Linux and MacOS) is:
```sh
LIGHTS=4 SHAKA="when the walls fell" ryan py_program.ryan
```
Or...
```sh
export LIGHTS=4
export SHAKA="when the walls fell"
ryan py_program.ryan
```

## Getting help

If you want to dig deeper into the CLI, you can use the `--help` command, like so:
```sh
ryan --help
```
This will print useful information on the available commands and options along with the current version of Ryan that you are using.


## Integrating into your program

If your language does not support Ryan natively yet, or if you don't wish to use the existing native support, you can use the Ryan CLI to generate the final JSON for you. All you need to do is:

* Spawn a subprocess `ryan <file>` and _capture_ its output, more specifically, its `stdout`.
* Check if the return status is `0`. This will indicate if any error happened.
* Decode de output using you languages support for JSON. The output is guaranteed to be valid.

All these steps are standard to most, if not all, modern programming languages and you should be able to easily implement them without any external library or resources.
