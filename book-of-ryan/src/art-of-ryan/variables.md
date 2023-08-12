# Variables

If you have ever been to a maths class, you know what a variable is. It's an identifier that refers to a value. For example,
```
x = 1
```
In this case, `x` is one. However, it can get more complicated than that
```
x = 2 + 3 * 4
```
The power of variables really shine when you have to use the result of a computation in more than one place. In this case, variables are great at saving you a great deal of trouble:
```
let x be 2 + 3 * 4 ...

... then this:
x * x + 2 * x + 3
... is much better than this:
(2 + 3 * 4) * (2 + 3 * 4) + 2 * (2 + 3 * 4) + 3
```

Like most programming languages, Ryan leverages variables to make referring to _results of expressions_ (the same ones we have encountered last chapter) a much more delightful thing than hitting Copy+Paste all the time.

## Simple bindings

This is how you declare a variable in Ryan:
```ryan
let a_guy = "Ryan";
```
What this line says is that `a_guy` refers to the value `"Ryan"` from now on. Now, you can start using it to build more complex stuff on top of it:
```ryan
let a_guy = "Ryan";
`Hello, ${a_guy}!`
```
This will evaluate to the text `"Hello, Ryan!"`. The construction `let ... = ...;` is called a _variable binding_, becaus it binds the value (`"Ryan"`) to an identifier (`a_guy`).

## Variable names

Variable names in Ryan are represented by text, but not all names are valid. Here are the rules you must follow to create a valid variable name:

* Variable names can only contain the characters:
    * `a-z` (lowercase letters)
    * `A-Z` (uppercase letters),
    * `0-9` (any digit) and...
    * `_` (the underscore). 
* Variable names cannot be in a list of _reserved keywords_. These are names that are already used for other things in Ryan. Some examples of this you have already found:
    * Values like `true`, `false` and `null`.
    * Control words like `if`, `then`, `else` and (out newest acquaintance) `let`.
The list of words is not big and does not contain many reasonable variable names. Ryan will warn you if have chosen an invalid name. Finding a different one should be an easy task (e.g., append a `_` to the end of your name).

Of course, even if a variable name is valid, it does not mean that it is a _good_ name. Here are some useful tips when naming your variables:

* Avoid one-letter names, like `x`, `y` and `i`.
* You may use more than one word for variable name. When doing so use `snake_case` or `camelCase` (either one is fine). `dontjustglueverythingtogetherbecauseitsdifficulttoread`.

The key is to be _expressive_ but to keep it _short_.

## Variables are immutable, but can be shadowed

In Ryan, all values are immutable. That means that there is no way of changing the value after it was assigned to a variable. This is not true in many programming languages. For example, in Python:
```python
x = "abc"
x = x + "def"
# x is the _same_ variable, but now is "abcdef"
```
In Ryan, there is something called _shadowing_, where you can do something like this:
```ryan
let x = "abc";
let x = x + "def";
// the second x is a different variable than the first x.
// you haven't changed `x`; you just recreated it.
```
In other words:
* In many programming languages, one can assign a value to a variable and then mess around with the value or even change it completely.
* In Ryan, there is no such a thing. When you redeclare a variable, you effectively destroy the old one and create the new one from scratch.
The difference is subtle, but (sometimes) it matters. If you are new to the mutability-immutability, this might be too abstract to grasp at first, especially if you are relatively new to the programming business. If you don't get it, don't worry: it's not a big deal. There are few points where it _really_ matters and it will be pointed out explicitly.
