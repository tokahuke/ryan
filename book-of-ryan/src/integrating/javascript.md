# JavaScript

There are two ways you can use Ryan with JavaScript: you can use Ryan in a Web environment or you can use Ryan with NodeJS in your computer.

## Ryan on the Web

To install Ryan for a Web environment, you will need to integrate it with a bundler, such as WebPack. To do so, just add the `ryan-lang-web` package to your project:
```bash
npm install ryan-lang-web
```
From there, you can `import` Ryan into your project, like so:
```js
import * as ryan from "ryan-lang-web";

var result = ryan.fromStr(`
    let x = "a value";
    
    {
        x,
        y: "other value" 
    }
`);

// result will be `{ "x": "a value", "y": "other value"}`
```

Since the Web doesn't have a filesystem nor environment variables, the import system for Ryan works differently in the browser than in other environments. Basically, if nothing is set, Ryan will run in _hermetic_ mode, i.e. with all imports disabled. However, a list of imports _can_ also be easily configured. For more information, see the [package homepage](https://www.npmjs.com/package/ryan-lang-web) in the NpmJS.org website.


## Ryan with NodeJS

By now, Ryan for NodeJS is a direct port from the Ryan for the Web package, via the magic of WASM. To add Ryan to your NodeJS project, just use npm:
```bash
npm install ryan-lang-node
```
Since this is a direct port, this package works just like described in the last section. Unfortunately, this includes the fact that Ryan for NodeJS does not understands the filesystem or environment variables. This is a limitation that will be resolved in a future release, hopefully sooner than later. This will entail rewriting as a proper native NodeJS extension, running native code, as opposed to running WASM.
