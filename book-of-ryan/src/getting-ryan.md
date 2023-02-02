# Getting Ryan

There are plenty of different ways to get Ryan. It really depends on what your setup is, since Ryan is designed to be embedded in larger applications, although it also works smootly as a standalone application. There are threrefore, two things that we refer to as Ryan:

1. The Ryan CLI: a program that you install in your computer. You pass a `.ryan` in and get `.json` out. 
2. The Ryan library: a dependency you add to you project that takes a `.ryan` from somewhere and spits out data that your programming environment can understand.

Depending on your usecase, you might need to install one of the above. Probably most people will want to install both.

## Getting the Ryan CLI

### Windows

Go to the [Ryan repository](https://github.com/tokahuke/ryan/releases) and download the `.exe.zip` files for the lastest release in your computer. Unzip it and move it to somewhere nice. Don't forget to rename the file to `ryan.exe` to keep things simple.

### Linux

#### First option

Go to the [Ryan repository](https://github.com/tokahuke/ryan/releases) and download the `.zip` files for the lastest release in your computer. Unzip it, rename the binary to `ryan` and move it to `/usr/loca/bin` (you will need `sudo`).

> Note: a one-line installer is in the backlog.

### If you have Cargo

If you have Cargo, just run:
```
cargo install ryan-cli
```
And you are done. You will not even need `sudo`!

### MacOS

By now, mac users will have to go through `cargo` to install the CLI. Just do
```
cargo install ryan-cli
```
And you are done.

> Note: a brew formula for Ryan is in the backlog.

## Getting the Ryan library

Depending on your language, you can install a binding to Ryan from your standard package manager:
```
cargo install ryan      # Rust
pip install ryan-lang   # Python
npm install ryan-js     # JavaScript (web)
````
If a binding is not available to your language, you can always use the Ryan CLI + you favorite JSON parser as a fallback. The Ryan CLI is already thought-out for this kind of programmatic interaction.
