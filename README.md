# rsk: the requirements tracking tool made for developers
**rsk** is a intended to be an easy to use tool for a someone at any skill
level or quality background to easily write and track their requirements.

## Tutorial
Once installed call `rsk -h` and see the help message. `rsk tutorial` will start the
interactive tutorial.

## Purpose
Requirements and design-documentation are probably the most important components of
writing quality software. Without writing out your requirements and tracking your 
design specifications, it can be very difficult to develop and maintain the product 
you were aiming to create. However, there are no open source tools (or proprietary 
tools for that matter) that make this process simple, easy and fun. **rsk** aims to 
do that by giving you a:

 1. simple text-based format to write your requirements in (TOML). This makes it
      easy to track your requirements with the rest of your project using standard
      revision control tools (git, hg, etc)
 2. workflow that is easy for developers to integrate with
 3. UI that is familar and useful -- helping the developer track their own progress
      from requirements to design to implementation and testing

It is hard to keep documentation up to date, especially when it doesn't aid
the core developer in tracking their progress. **rsk** aims to bridge that gap,
giving you a simple tool that you can use to track your project completeness and
test coverage.

If you like or use **rsk** please star:star: it on [github](https://github.com/vitiral/rsk) 
and mention it to frieds and colleauges. It is my belief that requirements tracking needs to 
be as second nature for developers as revision control and unit testing has become. I hope
that it will help us build better software and thus make the world a little bit better.

If you find bugs or have any suggestions, please open them here:
[bug tracker](https://github.com/vitiral/rsk/issues)

### Beta Notice
**rsk** is still in Beta and is not 100% feature complete. The API for the cmdline and
text format is expected to be stable, but the author reserves the right to change anything
that needs to be changed if it improves usability. 

## Installation

There are currently two installation options: downloading a cross-linux musl
binary or installing through Cargo.

### Linux Binary
Download [this file](https://github.com/vitiral/rsk/raw/master/target/x86_64-unknown-linux-musl/release/rsk) and put it's directory in your PATH variable.

**OR follow these directions**

Run this in the terminal to download the **rsk** binary in `~/bin`:
```
mkdir -f ~/bin && wget -q https://github.com/vitiral/rsk/raw/master/target/x86_64-unknown-linux-musl/release/rsk -O ~/bin/rsk && chmod a+x ~/bin/rsk
```

Now add `~/bin` to your `$PATH` in `.bashrc`. With a text editor, copy/paste this to the end:
```
export PATH="$HOME/bin:$PATH"
```

Now run `source ~/.bashrc`. You should now be able to run `rsk -h`. Update by simply re-running the 
first line.

### Cross Platform with [cargo](https://github.com/rust-lang/cargo)
> Note: neither windows or mac are tested. If you try either, whether it works or not,
> please open an [issue](https://github.com/vitiral/rsk/issues) to let us know!
If you have rust and cargo installed, simply execute:
```
cargo install rsk
```
and follow any directions it tells you to.
