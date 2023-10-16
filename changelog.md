
# v0.1.1

- Fixed a lot of typos.
- Added a ton of native patterns (iterators and strings utilities).
- Added the `in` operator for inclusion in lists, dicts and strings.
- Dictionary comprehensions.


# v0.2.0

- Make operations short-circuiting: `and`, `or` and `?` don't execute right side if not needed.
- Forgiving numbers: use `1_000` for large numbers.
- Variable key in dict: `let x = 1; { x }` works.
- Get rid of serde_json: now, we have a native `Deserializer` for Ryan.
- Quoting with snailquote is kinda... wrong: implemented correct JSON quoting.
- Remember insertion order, just like Python dictionaries.


# v0.2.1

- Template strings: `` `like ${"this"}` ``.


# v0.2.2

- Flatten expressions for lists: `[1, 2, ...[3, 4]] = [1, 2, 3, 4]`.
- Flatten expressions for dicts: `{a: 1, ...{b: 2, c: 3}} = {a: 1, b: 2, c: 3}`.
- Flatten expressions from list to dict and from dict to list.


# v0.2.3

- Fix list comprehensions bug inside patterns: pattern was not capturing.
- Multi-loop comprehensions.
- Type casting. Only `as int`, `as float` and `as text`, which will be enough for
most cases.
