# Not-so-simple types

_Collections_ are types which are used to aggregate and organize a set of data. Ryan supports two collection types: _lists_ and _dictionaries_ (or maps).

## Lists

Lists are a collection of values, sequentially ordered. Here are some lists for you:
```ryan
[1, 2, 3]       // Lists are a comma-separated sequence of values between brackets.
[1, "a", null]  // You can mix and match the types however you want.
[]              // An empty list is also a list
[
    1,
    2,
    3,  // Use forgiving commas for long lists
]
```
The two main operations on lists are _concatenation_ (just like with strings) and _index accessing_. Concatenation is pretty straightforward:
```ryan
[1, 2, 3] + ["a", "b", "c"]     // -> [1, 2, 3, "a", "b", "c"]
```
Index accessing is also easy: get the n-th element in the list. However, Ryan shares a pet-peeve with many other programming languages: the _first_ position is indexed by the number zero.
```
[1, 2, 3][0]        // -> 1
[1, 2, 3][1]        // -> 2, not 1!
[1, 2, 3][3]        // error! Tried to access index 3 of list of length 3 
                    // (3 is the 4th element!!)
```

## Dictionaries (or maps)

dictionaries are a collection of values indexed by strings. This name, dictionary, is quite apt in describing what it does. Just like the regular old book, it _uniquely_ associates a word to a value. Here is an example of a Ryan dictionary:
```ryan
{
    "name": "Sir Lancelot of Camelot",
    "quest": "to seek the Holly Grail",
    "favorite_color": "blue"
}
```
However, the same dictionary is much nicer written this alternative way:
```ryan
{
    name: "Sir Lancelot of Camelot",
    quest: "to seek the Holly Grail",
    favorite_color: "blue",     // use forgiving commas for extra niceness
}
```
Whenever the _key_ of a dictionary could be a _valid variable name_, you can omit the double quotes of the string. This doesn't change the value of dictionary (both examples correspond to the same thing); this is just _syntax sugar_: an nicer way of expressing the same thing.

Dictionaries have other few different tricks on their sleeves. For example, it is _syntactically valid_ to repeat a key in the dictionary:
```ryan
{
    a: 1,
    a: 2,
}
```
However, only the _last_ occurrence of the same key will count to the final result. The above dictionary evaluates to `{ a: 2 }`. 

You can also specify an _`if` guard_ at the end of each key, in order to make its insertion in the dictionary optional, like so:
```ryan
{
    a: 1,
    b: 2 if "abc" == "def",     // wrong!
    c: 3 if 1 == 1,             // quite true...
}
```
This will evaluate to `{ "a": 1, "c": 3 }`.

Lastly, just like with lists, you can concatenate dictionaries and index them in the very same fashion as you would do a list:
```ryan
let x = { a: 1, b: 2, c: 3 };
let y = { d: 4, e: 5, f: 6 };
x + y       // -> { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6 }
x["a"]      // -> 1
x["d"]      // error! Key "d" missing in map
```
However, following the same idea as before, if a certain key could be a _valid variable name_, one can also use the shorter `.` operator to index dictionaries:
```ryan
let x = { a: 1, b: 2, c: 3 };
x.a     // -> 1
a.d     // error! Key "d" missing in map
```
