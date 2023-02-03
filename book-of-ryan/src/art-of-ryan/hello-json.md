# Hello... Jason!?

> Note: if you are unfamiliar with JSON, you may skip this chapter and go right ahead to the next one. You will still be able to understand what Ryan is all about.

This tutorial is focused on getting you up-to-speed with Ryan as painlessly as possible. Where else then better than a good Hello world, eh?

```ryan
{
    "words": ["Hello", "World!"]
}
```

That's right! That is valid Ryan right there. The first thing you should know is that Ryan is a superset of JSON, that same format that grows like weed all over the internet. That is on purpose: Ryan aims to be uncomplicated and familiar. 

On the other hand, we all know that JSON can sometimes be a pain to edit. You can't even write a comment on that damn thing! And don't even get me started on multiline strings... Sometimes, JSON is a victim of its most important virtue: being dead simple.


## Simple amenities

Ryan, however, can be a bit more complex, trading performance for convenience. In fact, Ryan was made more for humans than for machines. Therefore, in Ryan, you can have stuff like this:

```ryan
// A comment!
```

Shocking, I know. Ryan only supports line comments, which start on a `//` and ends on a line break. There are no block comments, but why would you want one of those anyway? 

Oh!, by the way... that last line of Ryan was totally valid. I know it's a bit of a side note, but
```ryan

```
is a full ryan program in its own right and evaluates to `null`. 

On the same note, much stuff that you are used to have from JavaScript you can also have here:
```ryan
[
    "forgiving",
    "commas", 
]
```
and
```ryan
{
    naked: 1,
    dictionary: 2,
    keys: 3,
}
```
And finally, for everyone's delight, 
```
"multi
line
strings
are allowed!"
```
... given that they are valid UTF8. However, only double-quoted strings are possible. How could you possibly like those single-quotes aberrations?

## Conclusion to the introduction

You might be asking yourself about all the cosmetics and nice gadgets from the last section: "so what? JSON5 offers basically the same thing and YAML does that and _tons_ more. What is the upside here?" Well, Ryan is capable of much more than these simple tricks and that is what we are going to explore in the following chapters. However, if all you got from this book is that Ryan is a nicer JSON, well... that's a start (and _definitely_ a legitimate use for Ryan).
