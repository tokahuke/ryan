# Comprehensions

Comprehensions are a special kind of list and dictionary syntax that lets you make transformations on collections. This includes mapping and filtering values from these collections. If you are familiar with _Python_ list comprehensions, you will feel right at home, since the syntax is almost identical. If not, think of comprehensions as a controlled form of a `for` loop that looks like a set definition from mathematics.

## List comprehensions

A (basic) list comprehension takes the following form:
```ryan
[<expression> for <pattern> in <expression>]
```
For example, let's generate a list of even numbers:
```ryan
[2 * i for i in range [1, 10]]
```
This will evaluate to `[2, 4, 6, 8, 10, 12, 14, 16, 18]`. The expression `range [1, 10]` will generate all integer numbers from `1` to the _predecessor_ of `10` (i.e., `9`). If you find this strange, this is the default in most programming languages. Ryan is is not the exception. The comprehension expression binds the variable `i` to each element from the collection supplied after `in` and calculates `2 * i` for each value. 

You can also supply an optional `if` guard that will filter the elements that will make to the final collection. For example, to get the same result as before, instead of multiplying by `2`, we could iterate through all numbers and check the ones that are divisible by 2, like so:
```ryan
[i for i in range[1, 20] if i % 2 == 0]
```
This wil also yield `[2, 4, 6, 8, 10, 12, 14, 16, 18]` as the output.


## Dictionary comprehensions

Dictionary comprehensions are very similar to list comprehensions, the only difference being that you also get to set the keys od the dictionary as part of the comprehension:
```ryan
{<key expression>: <value expression> for <pattern> in <expression>}
```
So, for example, we could get a mapping from a number to its double, like so:
```ryan
{fmt i: 2 * i for i in range [1, 10]}
```
This will yield the dictionary `{"1": 2, "2": 4, "3": 6, "4": 8, "5": 10, "6": 12, "7": 14, "8": 16, "9": 18}`. Similarly to list comprehensions, you can also supply an optional  `if` guard to filter the values:
```ryan
{ fmt i / 2: i for i in range[1, 20] if i % 2 == 0 }
```
This will yield the same dictionary as before.


## What can go after a `for ... in`

Things that can go after the `in` keyword (also called iterables) are by now only lists and dictionaries. In the case of dictionaries, the patter in the `for` will be matched to the tuples of keys and values in the dictionaries, like so:
```ryan
{ y: x for [x, y] in {"a": "b", "c": "d"} }
```
This will yield the value `{"b": "a", "c": "d"}` as a result.

As you can see, there are also some handy patterns that can help you with some usual iterating tasks. We have already encountered `range`, that returns lists of consecutive numbers, but there are three more useful patterns that always come in handy:

* `enumerate`: returns pairs of the _index_ of an element and the element of the iterable, like so:
```ryan
enumerate [1, 4, 6, 9]      // -> [[0, 1], [1, 4], [2, 6], [3, 9]]
```
* `zip`: walks through a list of iterables in lockstep, like so:
```ryan
zip [[1, 2, 3], [4, 5, 6]]  // -> [[1, 4], [2, 5], [3, 6]]
```
* `sort`: returns a sorted version of a list:
```ryan
sort [1, 4, 3, 2]       // -> [1, 2, 3, 4]
```
