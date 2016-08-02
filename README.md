# rsk: the requirements tracking tool made for developers
**rsk** is a intended to be an easy to use tool for a someone at any skill
level or quality background to easily write and track their requirements.

## Tutorial
Once installed call `rsk -h` and see the help message. `rsk tutorial` will start the
interactive tutorial.

## Purpose
Requirements and design-documentation are probably the most important components of
writing quality software. Without them it can be very difficult to develop and
maintain the product you were aiming to create. However, there are no open source
tools (or proprietary tools for that matter) that make this process simple, easy
and fun. **rsk** aims to do that by giving you a:

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
and mention it to friends and colleagues. It is my belief that requirements tracking needs to
be as second nature for developers as revision control and unit testing has become. I hope
that by building better software we can all make the world just a little bit better.

If you find bugs or have any suggestions, please open them here:
[bug tracker](https://github.com/vitiral/rsk/issues)

### Beta Notice
**rsk** is still in Beta and is not 100% feature complete. The API for the cmdline and
text format is expected to be stable, but the author reserves the right to change anything
that needs to be changed if it improves usability.

## Installation

There are currently two installation options: downloading a cross-linux
binary or installing through Cargo.

### Linux Binary
Download [this file](https://github.com/vitiral/rsk/raw/master/target/x86_64-unknown-linux-musl/release/rsk) `chmod a+x` it, and put it in a directory on your `$PATH`

**OR follow these directions**

Run this in the terminal to download the **rsk** binary into `~/bin`:
```
mkdir -p ~/bin && wget -q https://github.com/vitiral/rsk/raw/master/target/x86_64-unknown-linux-musl/release/rsk -O ~/bin/rsk && chmod a+x ~/bin/rsk
```

Now add `~/bin` to your `$PATH` in `.bashrc`. With a text editor, copy/paste this to the end:
```
export PATH="$HOME/bin:$PATH"
```

Now run `source ~/.bashrc`. You should now be able to run `rsk -h` from anywhere. Update **rsk**
by simply re-running the first line.

### Cross Platform with [cargo](https://github.com/rust-lang/cargo)
> Note: neither windows or mac are tested. If you try either, whether it works or not,
> please open an [issue](https://github.com/vitiral/rsk/issues) to let us know!
If you have rust and cargo installed, simply execute:
```
cargo install rsk
```
and follow any directions it tells you to.

# Ultra Simple Tutorial
> **For a full tutorial, install rsk and run `rsk tutorial`**

If I was writing a "hello world" program in python and wanted to track requirements,
this would be the process.

 1. `mkdir` and `cd`to an empty folder
 2. `rsk init`
 3. `mkdir docs`
 4. `vim docs/design.rsk` and write my requirements and design
```
# docs/design.rsk
[REQ-world]
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
text = '''
there **shall also** be a way to say hello to aliens, but
that will be harder
'''

[SPC-aliens]
text = '''
I think we should use SETI or something
'''
```
 5. `mkdir src`
 6. `vim src/hello.py` and write my program
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
 7. `python2 src/hello.py`: it says hello world! That is good design :)
 8. `vim ~/.rsk/settings.rsk` and add `"{repo}/src"` to `code_paths`
 9. `rsk ls -L` to show this lovely status report (run on your own computer for color):
```
|--|    50%    50% | REQ
|--|     0%     0% | REQ-aliens
|DT|   100%   100% | REQ-world
|--|    50%    50% | SPC
|--|     0%     0% | SPC-aliens
|DT|   100%   100% | SPC-world   | <L:$CWD/src/hello.py(12:16)>
|DT|   100%   100% | TST
|DT|   100%   100% | TST-world   | <L:$CWD/src/hello.py(8:16)>
```

As you can see, we've finished our specs and tests for saying hello to the world,
but not to the aliens. It would be good to know if you had finished the second part
when the aliens arrive and **rsk** can help you do that!

# Future development
Here is a snapshot (0.1.0) of items yet to be started:
```
$ rsk ls -c '<' -T
|--|     0%     0% | REQ-1-interop            | rsk **will** provide simple methods for other tools to interact with it's data
|--|     0%     0% | REQ-1-scale              | rsk **will** be able to handle scale from very small projects with a single design document to enourmous multi-project multi-folder projects.
|--|     0%     0% | REQ-2-interop            |
|--|     0%     0% | REQ-2-interop-json       | rsk **will** provide json export utility for other tools to utilize.
|--|     0%     0% | REQ-2-performance        | rsk **will** aim to be as performant as is "reasonable" in both memory and cpu usage.
|--|     0%     0% | REQ-2-performance-store  | rsk **will** use a serialized file to speed up processing of data. Once a file has been processed, it's **will** be able to be loaded from the file instead of re-parsed.
|--|     0%     0% | REQ-2-ui-markdown        | when displaying to the user, all text fields **will** be processed as a simple markdown format
|--|     0%     0% | REQ-2-ui-tutorial        | rsk **will** provide an interactive tutorial to learn rsk as well as the basics of why to use requirements tracking in the first place.
|--|     0%     0% | REQ-2-ui-web             | rsk **will** provide a HTTP web server which can host up-to-date requirements as well as provide a REST-JSON API server for tracking test execution over a period of time.
|--|     0%     0% | REQ-status               | The `status` command gives the user information on any errors that exist in the project artifacts such as:
|--|     0%     0% | REQ-tutorial             | There **shall** be a tutorial that helps the user learn about rsk.
```


If you want to see items that have been mostly complete but have not been tested,
clone this repo and run:
```
rsk ls -c '>50' -t '<99' -T
```
