[![Build Status](https://travis-ci.org/vitiral/rst.svg?branch=master)](https://travis-ci.org/vitiral/rst)
# rst: the requirements tracking tool made for developers

[![Join the chat at https://gitter.im/rst-app/Lobby](https://badges.gitter.im/rst-app/Lobby.svg)](https://gitter.im/rst-app/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
**rst** is a [requirements tracking](https://en.m.wikipedia.org/wiki/Software_requirements_specification)
tool made for developers. It is an acronym for "Requirements, Specifications and Tests".

**rst** is pronounced like "wrist"

If you have any questions, [open an issue](https://github.com/vitiral/rst/issues) or
[chat in gitter](https://gitter.im/rst-app/Lobby)

## Tutorial
Once installed run `rst -h` on the cmdline to view the help message. `rst tutorial`
will start the interactive tutorial.

## Purpose
Requirements and design documentation are probably the most important components of
writing quality software. Without them it can be very difficult to develop and
maintain the product you were aiming to create. However, there are no open source
tools (or proprietary tools for that matter) that make this process simple, easy
and fun. **rst** aims to do that by giving you a:

 1. simple text-based format to write your requirements in
      ([TOML](https://github.com/toml-lang/toml)). This makes it easy to track
      your requirements with the rest of your project using standard revision
      control tools (git, hg, etc) and code reviews
 2. workflow that is easy for developers to integrate with and a web-ui to allow
      team collaboration and presentations.
 3. UI that is familiar and useful -- helping the developer track their own progress
      from requirements -> design -> implementation -> testing

It is hard to keep documentation up to date, especially when it doesn't aid
the core developer in tracking their progress. **rst** aims to bridge that gap,
giving you a simple tool that you can use to track your project completeness and
test coverage.

If you like or use **rst** please star:star: it on
[github](https://github.com/vitiral/rst) and mention it to friends and colleagues.
It is my belief that requirements tracking needs to be as second nature as revision
control and unit testing has become. I hope that by building better software we can
all make the world just a little bit better.

If you find bugs or have any suggestions, please open an issue here:
[bug tracker](https://github.com/vitiral/rst/issues)

### Beta Notice
**rst** is still in Beta and is not 100% feature complete. The API for the cmdline and
text format is expected to be stable, but the author reserves the right to change anything
that needs to be changed if it improves usability.

**Future improvements include:**
 - Additional command line tools
     - export: export artifacts to json, csv, html and other formats
 - Test Tracking: REST API with DB backend for tracking test execution
     plus cmdline utility and webui for viewing test execution.
     (rst currently only supports tracking implementaiton, not execution)
 - The Web UI is currently read-only. It will be able to edit soon.

# Installation

rst is compiled for linux, mac and windows. You can find releases on the
**[github release page](https://github.com/vitiral/rst/releases)**.

For Linux and Mac simply download and unpack the tarball with
`tar -zxvf RELEASE.tar.gz`. Then put it somewhere in your
[PATH](http://unix.stackexchange.com/questions/26047/how-to-correctly-add-a-path-to-path)

For Windows, simply download the zip, unzip it and run `./rst.exe` via git-bash
or some other linux emulator. It seems that the \*windows-gnu.zip works the best
for windows10.

## Installing with [cargo](https://github.com/rust-lang/cargo)

- First, install rust and cargo with [rustup](https://github.com/rust-lang-nursery/rustup.rs)
- If you want the web-ui, also install [node.js](https://nodejs.org/en/) by downloading or
    via your package manager
- If you want the web-ui, run `npm install -g elm webpack`
- clone the source code and cd: `git clone https://github.com/vitiral/rst.git; cd rst`
- compile the code: `cargo build --release --features web`

> Note: you can leave of "--features web" if you only want the command line ui

> Note: on windows, you will have to manually compile the javascript code.
> Simply `cd web-ui; npm run build` and it will be compiled.

# Ultra Simple Tutorial
> **For a full tutorial, install rst and run `rst tutorial`**

If I was writing a "hello world" program in python and wanted to track requirements,
this would be the process.

 - `mkdir` and `cd`to an empty folder
 - `rst init`
 - `vim reqs/design.toml` and write my requirements and design
```
# reqs/design.toml
[REQ-purpose]
text = '''
we need to be able to say hello to both the world and aliens
'''

[REQ-world]
partof = "REQ-purpose"
text = '''
there **shall** be way to say hello to the world. All of it.
'''

[SPC-world]
text = '''
The hello-world function shall say hello by printing it on
the cmdline
'''

[TST-world]
text = '''
To make this testable, there will be an intermediary
function that can test it before printing.
'''

[REQ-aliens]
partof = "REQ-purpose"
text = '''
there **shall also** be a way to say hello to aliens, but
that will be harder
'''

[SPC-aliens]
text = '''
I think we should use SETI or something
'''
```
Okay, now that we've written our requirements and design, let's start coding!
 - `mkdir src`
 - `vim src/hello.py` and write my program
```
#!/usr/bin/python2

def _hello_world():
    ''' an intermediary function to allow for testing '''
    return "hello world!"

def test_greeting():
    ''' test that the greeting works as expected
    partof: #TST-world '''
    assert _hello_world() == "hello world!"

def hello_world():
    ''' say hello to the world
    partof: #SPC-world '''
    print _hello_world()

if __name__ == '__main__':
    test_greeting()
    hello_world()
```
 - `python2 src/hello.py`: it says hello world! That is good design :)
 - `vim ~/.rst/settings.toml` and add `"{repo}/src"` to `code_paths`
 - `rst ls` to show this lovely status report

![rst ls example](http://i.imgur.com/GrDFLxr.png?1)

 - `rst check` to validate that there are no errors
 - `rst server` to host my requirements on a server and view it via my browser

As you can see, we've finished our specs and tests for saying hello to the world,
but not to the aliens. If the aliens arrived, it would be nice to know whether you
can say hello -- **rst** can help you do that!

# Future development
Here is a snapshot (0.1.0) of items yet to be started:
```
$ rst ls -c '<' -T
```

 - REQ-1-interop            : rst **will** provide simple methods for other tools to interact with it's data
 - REQ-1-scale              : rst **will** be able to handle scale from very small projects with a single design document to enourmous multi-project multi-folder projects.
 - REQ-2-interop            :
 - REQ-2-interop-json       : rst **will** provide json export utility for other tools to utilize.
 - REQ-2-performance        : rst **will** aim to be as performant as is "reasonable" in both memory and cpu usage.
 - REQ-2-performance-store  : rst **will** use a serialized file to speed up processing of data. Once a file has been processed, it's **will** be able to be loaded from the file instead of re-parsed.
 - REQ-2-ui-markdown        : when displaying to the user, all text fields **will** be processed as a simple markdown format

If you want to see items that have been mostly complete but have not been tested,
clone this repo and run:
```
rst ls -c '>50' -t '<99' -T
```

# Licensing
The rst file format (the format of the toml files, artifact name, etc) is
licensed under the CC0 (Creative Commons Public Domain) License. Any person can
use the format for any reason without the need for even attribution (attribution
is appreciated though!)

The rst library and web-ui are licensed under the LGPLv3+, except for files
which say otherwise in their header. See LICENSE.txt for more information.

