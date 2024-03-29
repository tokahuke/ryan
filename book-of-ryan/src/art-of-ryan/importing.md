# Importing things

Keeping with the theme of reusing code that we introduced when talking about [variables](./variables.md), Ryan also you to use a whole `.ryan` file as a kind of variable in other `.ryan` files. Imagine that you have a big projects in which you need to have many `.ryan` files for many different purposes. Normally, it's the case that you need to share a couple of variables between all these files, for example a username or a link to a shared resource. It would be a pain to have to redeclare it in every file, not to say dangerous if you ever need to change its value.

To cope with this scenario, you could create a `common.ryan` with all the repeated values and then, in each file, just do
```ryan
let common = import "common.ryan";
```
This will evaluate the contents of `common.ryan` and put it in the variable `common`. The file `common.ryan` must be in the same directory as the file containing the import statement, otherwise Ryan won't be able to find it. If your file is in another path, you can use your operational system's path system to locate it:
```ryan
let common = import "../other-stuff/common.ryan";       // e.g, in Linux and MacOS
let common = import "C:\Users\ryan\stuff\common.ryan";  // e.g, in Windows
```

## Customizing your files

Another very common use-case for imports is to customize the JSON generated by your `.ryan` depending on the environment it is executed in. For example, it's very common for programs to be configured differently when testing than when put to run "for real" (also called the _production_ environment). Usernames, passwords and resource names will be completely different to avoid a rogue test to ruin the operation of your system.

Enter _environment variables_: these are a set of variables that your operational system passes to every program in order to customize its behavior. Of course, you can set these variable directly through the command line:
```
MY_VAR=1 ryan < my_file.ryan
```
This will invoke `ryan` on `my_file.ryan` with `MY_VAR` set to 1. You can then access this variable from your `.ryan` file as so:
```ryan
let my_var = import "env:MY_VAR";   // my_var will be set to `1`.
```
You can pass whole Ryan programs in environment variables, if you wish, although it may not be the most comfortable thing to do. The only restriction is that such programs can only access other environment variables; it cannot touch your files anymore. Therefore, this will not work:
```ryan
MY_PROG='import "common.ryan"' ryan < my_file.ryan
```
If `my_file.ryan` tries to `import "env:MY_PROG"`, an error will be raised.

## Importing chunks of text

Up to now, we have only talked about importing Ryans from Ryans. However, in many cases, it is very quite to import text directly, verbatim. Ryan saves you the trouble of writing quotations and escape sequences by allowing you to import things `as text`:
```ryan
let gilgamesh = import "tale-of-gilgamesh.txt" as text;
```
You can also use this with environment imports:
```ryan
let username = import "env:USERNAME" as text;       // username is `"Ryan"`.
```
If `USERNAME` is set to `Ryan`, without the `as text`, you would get an error: after all the _variable_ `Ryan` has not been set in your `USERNAME` program. With the `as text`, Ryan will understand that we only want the string `"Ryan"`.

## Setting defaults

If the imported file does not exist or the environment variable is not set, Ryan will, by default, raise an error. You can provide a default value to override this error using `or`:
```ryan
import "env:FORGOT_TO_SET" or "Ryan";   // -> "Ryan"
import "does-not-exist.ryan" or {};     // -> (empty dictionary)
```
The clause `or` will force the import to use the default value if, _for any reason whatsoever_ the import fails.

## Limitations

### No dynamic imports

Although `import` expects a string as input, you cannot use an expression that yields a string; the import must be only a literal string. This will not work:
```ryan
import "my-" + "file.ryan"
```
nor will this
```
let num = 4;
import `file-${num}.ryan`
```
You can however, go around this limitation in many cases. For example, you can use `if ... then ... else ...` for conditional imports:
```ryan
if 1 == 1 then
    import "this.ryan"
else
    import "that.ryan"
```
Even though `import` does not accept expressions, it can be freely used within expressions to allow for some level of customization.


### No circular imports

Circular imports will result in a nasty, smelly error:
```ryan
// tweedledee.ryan:
import "tweedledum.ryan"

// tweedledum.ryan
import "tweedledee.ryan"
```
If you ever find yourself in this situation, you will need to restructure your files in order to destroy the cyclic dependency. If `a` and `b` depend on each other, you can to put the "depended" part in a third file `c` and make both `a` and `b` depend on this file instead. This "third file trick" solves most, if not all, situations you might encounter.
