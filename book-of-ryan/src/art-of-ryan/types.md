# Types

Up to now, we have talked about different types of values in Ryan: integers, boolean, string, dictionaries, etc... All these value types have their own representation in Ryan. Even though Ryan is not a _statically typed_ language, you can use types (and expressions over types) to add extra checking to your Ryan code, ensuring that it runs correctly. Furthermore, you can use _type guards_ to further refine patterns, either making them more strict or also allowing them to react differently for different types of data (also known as _polymorphism_). In this chapter, we will explore what Ryan can offer you in terms of typing.

## Primitive types

These are the primitive types in Ryan (you have encountered all them before; we are just giving them Ryan names):

* `bool`: booleans; can only be `true` or `false`.
* `int`: an integer, like `123` or `-4`, but *not* fractional numbers such as `1.2`.
* `float`: a floating point like `1.23` or `6e23`. This also includes `1.0`, which, although integer, is _stored_ and _processed_ as a float.
* `number`: `int` or `float`. Includes `123`, `1`, `1.0` and all other numerical stuff.
* `text`: strings of text, such as `"Ryan"`.
* `null`: the value `null`. Only `null` is of type `null`.
* `any`: anything goes!

## Composite types

With lists and dictionaries, Ryan gives you a bit more of flexibility than ony a set of flavours. The most simple types are "collections of the same kind of stuff":

* `[T]` (where `T` is another type): a list where _all_ elements are of type `T`. E.g., `[int]` is "a list of integer numbers".
* `{T}` (where `T` is another type): a dictionary where _all_ values are of type `T`. E.g., `{text}` is "a dictionary where the values are text".
* `?T` (where `T` is another type): either something of type `T` or `null`. This is called an _optional_ type. E.g., `?float` is "either a float number or 'nothing', i.e., `null`".

However, in a manner similar to patterns, you can also specify the nature of the elements:

* `[A, B, C]` (where `A`, `B` and `C` are other types): a list of _exactly_ the specified types (also called a tuple). E.g., `[int, bool]` is "a list of two elements where the first is an integer and the other is a boolean.
* `{a: A, "b": B}` (where `A` and `B` are other types): a dictionary with _exactly_ the specified keys whose values correspond to the specified types. E.g., `{a: int, b: bool}` is "a dictionary with exactly the `"a"` and `"b"` keys where the value for `"a"` is an integer and the value for `"b"` is a boolean.
* `{a: A, "b": B, ..}` (where `A` and `B` are other types): a dictionary with _at least_ the specified keys whose values correspond to the specified types. E.g., `{a: int, b: bool}` is "a dictionary with _at least_ the `"a"` and `"b"` keys where the value for `"a"` is an integer and the value for `"b"` is a boolean.


## Alternative types

In addition to the types we have already seen, you can further compose types with the alternative operator `|`.  The expression `A | B` means "something that can be either of type `A` or of type `B`". In fact, we have already found two _syntax sugars_ for this operation in the previous sections:

* `number` is the same thing as `int | float`.
* `?T` is the same thing as `T | null`.

However, you can create your own alternative types, e.g.:

* `[int | text]`: a list where the elements can be either integer or text.
* `int | {bool}`: an integer or a dictionary of booleans.


## Type guards

Type guards are an element of the Ryan pattern matching system that will only accept the pattern if, when binding a variable to a value, the value is of the specified type. Type guards are defined with `:`, like so:
```ryan
let x: int = 1;     // Success! `x` will be equal to 1
let x: int = "1";   // Error! Expected an integer, got text.
```
Every time you declare a new variable in a pattern match, you can define an optional type guard. If none is provided, it's assumed that the type is `any`, i.e., anything goes.

Things get fun when you can bring _polymorphism_ to your pattern matches by powering type guards with _alternative patterns_:
```ryan
let foo x: int = `I am an integer: ${x}`;
let foo x: float = `I am a float: ${x}`;
[foo 1, foo 1.0]        // -> ["I am an integer: 1", "I am a float: 1"]
```

It's recommended that you use type guards wherever possible. It helps keeping your code more _explicit_ on what is going on. Besides, it is one extra way to check the data your program is receiving. For example, suppose you want to set a debug level for your program, which is a number, like:
1. Only log errors
2. Log errors and high-level information
3. Log every small detail in the code execution (also known as verbose).
You can validate the input from an environment variable, like so:
```ryan
let debug_level: int = import "env:DEBUG_LEVEL";
```
This disallows people from passing `DEBUG_LEVEL=off` to your program and get a valid configuration, which could save you lots of pain down the line.

## Type aliases

Finding yourself writing the same long type over and over again? Fear not! Ryan supports _type aliases_. Type aliases are variable bindings that associate a variable to a given type. These are a bit different from the regular bindings in which they do _not_ allow destructuring with patterns and they must start with the `type` keyword, like so:
```ryan
type X = { a: int, very, text, long: {int}, type_expression: null };
```
Using regular `let` biding wont work, because in Ryan type expression are different from regular value expression (and they don't mix!):
```ryan
let X = int;    // -> error! Expected expression block, but got a type :(...
```
After you have defined a type alias, you can use it normally as if it were any other type:
```ryan
type X = int;
let x: X = 1;   // -> ok!
```
Remember that these are only type _aliases_. Type aliases do not declare a new type. Therefore, a same variable can conform to many different type aliases at the same type.


## Types are not representable

As you can expect, types have no equivalent in JSON. Therefore, even though types are values, if you ever sneak a Ryan type into a value to be represented in JSON, you will get a "not representable" error. By now, the only way to trigger this error is through the misuse of type aliases:
```ryan
type X = int;
{
    a_type: X,      // -> Un-representable value: int
}
```
