# rst: requirements as easy as abc
**rst** is a [requirements tracking](https://en.m.wikipedia.org/wiki/Software_requirements_specification) 
tool made for developers. It is an acronym for "Requirements, Specifications and Tests". 

**rst** is pronounced like "wrist"

## Tutorial
Once installed run `rst -h` on the cmdline to view the help message. `rst tutorial` 
will start the interactive tutorial.

## Purpose
Requirements and design-documentation are probably the most important components of
writing quality software. Without them it can be very difficult to develop and
maintain the product you were aiming to create. However, there are no open source
tools (or proprietary tools for that matter) that make this process simple, easy
and fun. **rst** aims to do that by giving you a:

 1. simple text-based format to write your requirements in 
      ([TOML](https://github.com/toml-lang/toml)). This makes it easy to track 
      your requirements with the rest of your project using standard revision 
      control tools (git, hg, etc)
 2. workflow that is easy for developers to integrate with
 3. UI that is familar and useful -- helping the developer track their own progress
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
     - status: analyze artifacts and display any errors and a summary
     - export: export artifacts to json, csv, html and other formats
     - tests: cmd for viewing and querying executed tests in relationship 
         to defined tests (see Test Tracking)
 - Test Tracking: REST API with DB backend for tracking test execution
     (rst currently only supports tracking implementaiton, not execution)
 - Web UI frontend to make it easy for non-devs to view requirements

## Installation

There are currently two installation options: downloading a cross-linux
binary or installing through Cargo.

### Linux Binary
All we will be doing is downloading [this file](https://github.com/vitiral/rst/raw/master/target/x86_64-unknown-linux-musl/release/rst), 
`chmod a+x` it, and putting it in a directory on your `$PATH`

If you know what all this means, just do that.

**OR follow these directions**

Run this in the terminal to download the **rst** binary into `~/bin`:
```
mkdir -p ~/bin && wget -q https://github.com/vitiral/rst/raw/master/target/x86_64-unknown-linux-musl/release/rst -O ~/bin/rst && chmod a+x ~/bin/rst
```

Now add `~/bin` to your `$PATH` in `.bashrc`. With a text editor, copy/paste this to the end:
```
export PATH="$HOME/bin:$PATH"
```

Now run `source ~/.bashrc`. You should now be able to run `rst -h` from anywhere. Update **rst**
by simply re-running the first line.

### Cross Platform with [cargo](https://github.com/rust-lang/cargo)
> Note: neither windows or mac are tested. If you try either, whether it works or not,
> please open an [issue](https://github.com/vitiral/rst/issues) to let us know!

If you have rust and cargo installed (recommendation: install them with
[rustup](https://github.com/rust-lang-nursery/rustup.rs)), simply execute:
```
cargo install rst_app
```
and follow any directions it tells you to.

> Note: this library was renamed to rst_app to not conflict with a future ReStructredText
> library. See [#6](https://github.com/vitiral/rst/issues/6)

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
The hello-world function shall say hello by printing it on the cmdline
'''

[TST-world]
text = '''
To make this testable, there will be an intermediary function that can
test it before printing.
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
 - `mkdir src`
 - `vim src/hello.py` and write my program
```
#!/usr/bin/python2

def _hello_world():
    ''' an intermediary function to allow for testing '''
    return "hello world!"

def test_greeting():
    ''' partof: #TST-world '''
    assert _hello_world() == "hello world!"

def hello_world():
    ''' partof: #SPC-world '''
    print _hello_world()

if __name__ == '__main__':
    test_greeting()
    hello_world()
```
 - `python2 src/hello.py`: it says hello world! That is good design :)
 - `vim ~/.rst/settings.toml` and add `"{repo}/src"` to `code_paths`
 - `rst ls -L` to show this lovely status report (run on your own computer for color):
```
|--|    50%    50% | REQ
|--|     0%     0% | REQ-aliens
|--|    50%    50% | REQ-purpose
|DT|   100%   100% | REQ-world
|--|    50%    50% | SPC
|--|     0%     0% | SPC-aliens
|DT|   100%   100% | SPC-world   | <L:$CWD/src/hello.py(12:16)>
|DT|   100%   100% | TST
|DT|   100%   100% | TST-world   | <L:$CWD/src/hello.py(8:16)>
```

As you can see, we've finished our specs and tests for saying hello to the world,
but not to the aliens. It would be good to know if you had finished the second part
when the aliens arrive and **rst** can help you do that!

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
 - REQ-2-ui-tutorial        : rst **will** provide an interactive tutorial to learn rst as well as the basics of why to use requirements tracking in the first place.
 - REQ-2-ui-web             : rst **will** provide a HTTP web server which can host up-to-date requirements as well as provide a REST-JSON API server for tracking test execution over a period of time.
 - REQ-status               : The `status` command gives the user information on any errors that exist in the project artifacts such as:
 - REQ-tutorial             : There **shall** be a tutorial that helps the user learn about rst.

If you want to see items that have been mostly complete but have not been tested,
clone this repo and run:
```
rst ls -c '>50' -t '<99' -T
```
