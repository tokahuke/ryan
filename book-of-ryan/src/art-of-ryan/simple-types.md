# Simple types and simple operations

## Numbers

The simplest type you can think about in Ryan is a number. Numbers come in two flavors: _integers_ and _floats_. Both types are treated diffently, although interchangibly via type coercion (more on that later). Here are some examples of numbers:
```ryan
123     // an integer
1.23    // a float
123.0   // another float, not an integer
-12     // integers can be negative too
1e23    // and floats can be written in scientific notation if they are veery big
1e-42   // ... and also if they are veeery tiny
```
If you know other programming languages, this is most probably identical to what you have encoutered before. 

In Ryan, you can write arbitrary numerical expressions mixing and matching floats and integers. When necessary, Ryan will implicitly convert the integer to the corresponding float. This is called type coercion. Here are some example of what you can do:
```ryan
2 + 3           // add integers
2.0 + 3         // mix and match integers with floats freely
5 - 4 * 3 / 2   // arbitrary operations with the standard precedence
(2 + 3) * 4     // use parentheses
23.0 % 7.0      // modulo operation is supported, even for floats 
```

## Booleans

Booleans indicate a binary choice and come only on two values `true` or `false`. They can be operated upon using the three canonical operations `and`, `or` and `not`:
```ryan
true and false      // -> false
false or false      // -> false
not false           // -> true
```

Things get even more interesting when you combine test operations on integers and float to produce booleans:
```ryan
1 == 2              // -> false (tests for equality)
1 != 2              // -> true  (tests for inequality)
1 > 2               // -> false (tests if left is greater than right)
1 >= 2              // -> false (tests if left is greater or equal to right)
1 < 2               // -> true  (tests if left is less than right)
1 <= 2              // -> true  (tests if left is less or equal to right)
```

And, of course, you can match everything together to create complex boolean expressions:
```ryan
1 > 2 or 3 > 4          // -> false
not 1 == 2              // -> true
2 % 3 == 2 and 1 < 0    // -> false
```

### `if ... then ... else ...`

This construction can be used to control the value of an expression based on a condition. The `if` clause accepts an expression that returns a boolean and then the final result of the expression is the expression after `then` if `true` or `else` if `false`. Some examples are shown below:
```ryan
if 1 == 1 then 123 else 456     // -> 123
if 1 != 1 then 123 else 456     // -> 456
if 2 >= 3 then 123              // -> error! There always needs to be an `else`
if 0 then 123 else 456          // -> error! The `if` expression has to be a boolean 
```

## Strings

Strings are pieces of _escaped_ text that can represent any UTF-8 encodable data. They come between `"` (double quotes) and boast a good deal of escape characters, like `\n` (new line). If you come from other programming languages, the convention here is probably the same you are used to. Some examples of strings are shown below:
```ryan
"abc"       // -> abc
"ab\nc"     // -> ab<enter>c
"ab\"c"     // -> ab"c (`\"` is how you write a double quote without being ambiguous)
"multi
line
strings
are
welcome too"
'but single-quotes are not'     // -> error! only double quotes allowed
```

Strings can be added together for concatenation:
```ryan
"abc" + "def"       // -> abcdef
```
But you cannot add numbers and strings together to get the "intended" result:
```ryan
"there are " + 4 + " lights"    // -> error! Cannot add text and integer
```
However, you can use the "`fmt` trick" to achieve the desired result:
```ryan
"there are " + fmt 4 + " lights"    // there are 4 lights
```
The `fmt`... thingy... takes any value in Ryan and produces a string representation of it, even if it is not very useful.

> Note: `fmt` is called a native pattern match and will be introduced later.


## `null`

Lastly, but not least, there is the simplest type of all: null. Null has only one value: `null` and represents the abscence of something. Null is not a boolean or an integer, so it will not behave like, say `false` or `0`. Therefore, all these won't work:
```ryan
1 + null                                // error!
if null then "wrong" else "wronger"     // error!
not null                                // error!
null > 0                                // error!
"The answer is" + null                  // error!
```
Null is in fact its own unique thing. However you _can_ do much with `null` via the `?` operator. This operator allows you to provide a _default_ value in case some expression of yours, for some reason evaluated to `null`:
```
null ? 1    // -> 1
2 ? null    // -> 2
3 ? 2       // -> 3
```
