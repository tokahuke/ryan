# Getting Ryan

There are plenty of different ways to get Ryan. It really depends on what your setup is, since Ryan is designed to be embedded in larger applications, although it also works smoothly as a standalone application. There are therefore, two things that we refer to as Ryan:

1. The Ryan CLI: a program that you install in your computer. You pass a `.ryan` in and get `.json` out. 
2. The Ryan library: a dependency you add to you project that takes a `.ryan` from somewhere and spits out data that your programming environment can understand.

Depending on your usecase, you might need to install one of the above. Probably most people will want to install both.

## Getting the Ryan CLI

### One-liner (Linux, MacOS)

Copy and paste the following command in your favorite console:
```sh
curl -L -Ssf "https://raw.githubusercontent.com/tokahuke/ryan/main/install/$(uname).sh" \
    | sudo sh
```
You will need `sudo` rights to run the script (this installation is system-wide).

### Download the binary from GitHub (Linux, MacOS, Windows)

Go to the [Ryan repository](https://github.com/tokahuke/ryan/releases/latest) and download the zipped file corresponding to your platform. Unzip it and move it to somewhere nice!

### Using `cargo`

If you have Cargo, just run:
```
cargo install ryan-cli
```
And you are done. You will not even need `sudo`!

## Getting the Ryan library

Depending on your language, you can install a binding to Ryan from your standard package manager:
```
cargo install ryan      # Rust
pip install ryan-lang   # Python
npm install ryan-js     # JavaScript (web)
````
If a binding is not available to your language, you can always use the Ryan CLI + you favorite JSON parser as a fallback. The Ryan CLI is already thought-out for this kind of programmatic interaction.
