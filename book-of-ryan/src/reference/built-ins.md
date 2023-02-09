# List of built-ins

> Note: some of these built-ins will be released in the 0.1.1 (or 0.2.0) version of Ryan.

<table style="width: 100%">
    <tr>
        <td style="min-width: 30%"><strong>Pattern</strong></td>
        <td><strong>Description</strong></td>
    </tr>
    <tr>
        <td><code>fmt x: any</code></td>
        <td>Transform any object into a string that represents it. Use this pattern to interpolate non-string values with string values in order to create more complex displays, e.g., <code>"there are " + fmt 4 + " lights"</code>. Without the <code>fmt</code>, you will get a type error.</td>
    </tr>
    <tr>
        <td><code>len x: [any] | {any} | text</code></td>
        <td>Gets the length of a list, a dictionary or a string.</td>
    </tr>
    <tr>
        <td><code>range [start, end]</code></td>
        <td>Generates a list of consecutive integer numbers from <code>start</code> to <code>end - 1</code>.</td>
    </tr>
    <tr>
        <td><code>zip [left, right]</code></td>
        <td>Iterates through both iterables at the same time, returning a list with the pairs of elements in the same position. For example, <code>zip [[1, 2, 3], [4, 5, 6]]</code> yields <code>[[1, 4], [2, 5], [3, 6]]</code>.</td>
    </tr>
    <tr>
        <td><code>enumerate x: [any] | {any}</code></td>
        <td>Generates a list of indexed value for a list. For example, <code>enumerate ["a", "b", "c"]</code> yields <code>[[1, "a"], [2, "b"], [3, "c"]]</code>.</td>
    </tr>
    <tr>
        <td><code>sum x: [number]</code></td>
        <td>Returns the sum of all numbers in a list.</td>
    </tr>
    <tr>
        <td><code>max x: [number]</code></td>
        <td>Returns the maximum of all numbers in a list.</td>
    </tr>
    <tr>
        <td><code>min x: [number]</code></td>
        <td>Returns the minimum of all numbers in a list.</td>
    </tr>
     <tr>
        <td><code>all x: [bool]</code></td>
        <td>Returns <code>true</code> if there is no <code>false</code> in the list.</td>
    </tr>
     <tr>
        <td><code>any x: [bool]</code></td>
        <td>Returns <code>false</code> if there is no <code>true</code> in the list.</td>
    </tr>
    <tr>
        <td><code>sort x: [number] | [text]</code></td>
        <td>Returns a sorted version of a list.</td>
    </tr>
    <tr>
        <td><code>keys x: {any}</code></td>
        <td>Returns the a list of the keys in the dictionary.</td>
    </tr>
    <tr>
        <td><code>values x: {any}</code></td>
        <td>Returns the a list of the values in the dictionary.</td>
    </tr>
    <tr>
        <td><code>split sep: text</code></td>
        <td>Returns the a pattern that splits a text by the supplied separator. Use it like so: <code>( split "," ) "a,b,c"</code> = <code>["a", "b", "c"]</code></td>
    </tr>
    <tr>
        <td><code>join sep: text</code></td>
        <td>Returns the a pattern that joins a list of text with the supplied separator. Use it like so: <code>( join "," ) ["a", "b", "c"]</code> = <code>"a,b,c"</code></td>
    </tr>
    <tr>
        <td><code>trim x: text</code></td>
        <td>Returns a text with all <em>leading</em> and <em>trailing</em> whitespaces removed.</td>
    </tr>
    <tr>
        <td><code>trim_start x: text</code></td>
        <td>Returns a text with all <em>leading</em> whitespaces removed.</td>
    </tr>
    <tr>
        <td><code>trim_end x: text</code></td>
        <td>Returns a text with all <em>trailing</em> whitespaces removed.</td>
    </tr>
    <tr>
        <td><code>starts_with prefix: text</code></td>
        <td>Returns a pattern that tests if a text starts with the given prefix. Use it like so: <code>( starts_with "foo" ) "foobar" </code> = <code>true</code></td>
    </tr>
    <tr>
        <td><code>ends_with postfix: text</code></td>
        <td>Returns a pattern that tests if a text ends with the given postfix. Use it like so: <code>( ends_with "bar" ) "foobar" </code> = <code>true</code></td>
    </tr>
    <tr>
        <td><code>lowercase x: text</code></td>
        <td>Makes all letters lowercase.</td>
    </tr>
    <tr>
        <td><code>uppercase x: text</code></td>
        <td>Makes all letters uppercase.</td>
    </tr>
    <tr>
        <td><code>replace [find: text, subst: text]</code></td>
        <td>Returns a pattern that substitutes all occurrences of the text <code>find</code> with the text <code>subst</code>. Use it like so: <code>( replace [ "five", "four" ] ) "There are five lights" </code> = <code>"There are four lights"</code></td>
    </tr>
    <tr>
        <td><code>parse_int x: text</code></td>
        <td>Parses some text as int, e.g<code>parse_int "123"</code> = <code>123</code>. This raises an error if the text is not a valid integer.</td>
    </tr>
    <tr>
        <td><code>parse_float x: text</code></td>
        <td>Parses some text as float, e.g<code>parse_float "123"</code> = <code>123.0</code>. This raises an error if the text is not a valid float.</td>
    </tr>
</table>
