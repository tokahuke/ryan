# Pattern matching

In this chapter, we will talk about the joys of breaking stuff up to its most basic components. 

## Destructuring bindings

Up to now, we have only worked with variable declarations (bindings) of the garden-variety format:
```ryan
let a_number = 123;
let a_text = "abc";
```
However, Ryan allows you to _destructure_ the value after the `=` sign and bind more than one value to the destructured value. Think of destructuring as circulating parts of a value with a pen and giving the circulated parts names, like so:
```ryan
let [a, b, ..] = [1, 2, 3, 4, 5];
```
With this statement, `a` will be sat to 1 and `b` will be set to 2. The `..` matches the tail of the list. 

Here is a list of the simple pattern matches you can do in Ryan:
```ryan
let _ = "abc";              // wildcard: accepts anything and binds no variables
let x = 1;                  // identifier: matches a variable to a value
let 1 = 1;                  // matches a literal (and binds no variable)

let [a, b] = [1, 2]         // list match: matches all the elements of the list.
let [a, b, ..] = [1, 2, 3]; // head match: matched the first elements of a list.
let [.., a, b] = [1, 2, 3]; // tail match: matched the last elements of a list.

let {"a": b} = {"a": 1};    // strict dict match: matches all values of a dict (b = 1)
let { a } = { a: 1 };       // you can bind a key to a value directly too! (a = 1)
let { a, ..} = {"a":1,"b":2}// dict match: matches only the specified keys
```

Of course, if the pattern you specified cannot match the input value, you will get an error:
```ryan
let { a, b } = [1, 2, 3];   // boom!
```

However, the real fun of destructuring patterns is that they are recursive: you can mix and match them like russian dolls. Therefore, something like this is perfectly legal:
```ryan
let {
    a,
    b: [c, d, 3],
    "e": _,
    f: { g, h, ..}
} = {
    a: "jalad and darmok",
    b: [1, 2, 3],
    e: "how many lights?",
    f: { g: 4, h: 5, i: 6 },
}
```
and will create the variables `a = "jalad and darmok"`, `c = 1`, `d = 2`, `g = 4` and `h = 5` all in one go. You can think of destructuring as an alternative and visual way of accessing values from lists and dictionaries.

## Pattern matches

Pattern matches power the patterns whe have just presented to create dependent execution, what folks in other languages call _functions_. A pattern match is a piece of Ryan code that can be applied to a value in order to produce another one. For example:
```ryan
let foo x = x + 1;
[foo 1, foo 2, foo 3]      // same as [1 + 1, 1 + 2, 1 + 3]
```
This code will evaluate to `[2, 3, 4]`. The pattern `foo` will substitute `x` by each of the provided values and evaluate the expression for each specific case.

In Ryan, pattern matches only take _one_ argument as input, as opposed to many languages out there that take more than one. However, this is by no means a limitation, because patterns!
```
let sum_both [a, b] = a + b;
sum_both [5, 7]     // -> 12
```
You can use any pattern to declare a pattern match.

## Closures

All pattern matches in Ryan are _closures_. That means that you are free to use variables defined outside the pattern match definition in your return expression:
```ryan
let object_name = "lights";
let there_are quantity = "There are " + fmt quantity + " " + object_name;

there_are 4     // -> "There are 4 lights"
```

## Locals

A pattern match does not expect only an expression, but a whole block. This means that the body of a pattern match can be its whole self-contained Ryan program, with its own local variables, imports, pattern matches, etc...
```ryan
let there_are quantity = 
    let object_name = "lights";
    "There are " + fmt quantity + " " + object_name;

there_are 4     // -> "There are 4 lights"
```

## Patterns are values too

Yep, patterns are values just like any other! Every time you define a pattern match with a `let`, you create a variable with that pattern match as a value:
```ryan
let foo x = x + 1;
foo     // -> ![pattern foo x]
```
You can even make a pattern match be the return value of another pattern match:
```ryan
let add a = 
    let do_add b = a + b;
    do_add;

(add 3) 2       // -> 5
```
The parentheses are needed here because pattern match application is _left-associative_ in Ryan.

The only limitation this equivalence is that pattern matches are not _representable_. Since they don't have a JSON equivalent, they cannot be converted to JSON. If the outcome of your Ryan program contains a pattern match anywhere, you will get an error. 

## Alternative patterns

The same pattern match can be defined multiple times with different patterns. Ryan will try to match the pattern in order until a match is found and execute the expression associated with the match:
```ryan
let foo 1 = 2;
let foo 2 = 10;
let foo x = x + 10;

[foo 1, foo 2, foo 3]   // -> [1, 10, 13]
```
This is very handy when defining special cases and can be used as a more visual alternative to `if ... then ... else ...`.

## Recursion is not allowed, in any case!

A pattern match cannot call itself in its code. This will not work:
```ryan
let foo x = [foo x];
foo 1
```
This would be an infinite program, that would never end! Thankfully, Ryan will complain that it cannot use the variable `foo` because it has not been declared before. Even if you try to declare it before, using alternative patterns, it will still not recurse:
```ryan
let foo [1] = 1;
let foo x = [foo [x]];    // "Now, `foo` is defined", says Will E. Coyote

foo 1   // -> [1]
```
As you can see, the _captured_ version of `foo` is different from the version we called in the end. Only the alternatives that existed up to the point of the pattern definition are captured.

Even though recursion is a nice clever trick without which we could not have computers as we know them, it would make Ryan too general for what it was initially conceived: make nice configuration files. It's not expected that people create enormously complex and sneaky algorithms in Ryan. Therefore, to force keeping things simple, no recursion allowed!
