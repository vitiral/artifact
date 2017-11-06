# SPC-cmd
partof: REQ-web
###
Specifications for the artifact cmdline interface

- [[SPC-cmd-tutorial]]: begin an interactive tutorial for the user
- [[SPC-cmd-init]]: initialize the current repository by creating `design/` and 
  `.art/settings.toml` files
- [[SPC-cmd-ls]]: filter and view artifacts
- [[SPC-cmd-check]]: check the project for errors
- [[SPC-cmd-fmt]]: format the `*.toml` files to be correct.
- [[SPC-cmd-export]]: export artifacts json and static html
- [[SPC-web]]: (`art serve` cmd) run an editable server for a full featured web 
  interface

# SPC-cmd-export
`art export` shall be able to export artifacts to various static file
formats. This will allow a user to display their artifacts in a pretty
format on services like github.

Syntax: `art export [-o|--output PATH] TYPE`

Supported types **shall** be:
- html: this is the MUST HAVE export type
- markdown: this is a possible future type

Flags:
- `-o` specifies the a different output directory frm the cwd to export
  the files

# SPC-cmd-fmt
`art fmt` will be the command that is run to format all files in a project
to their correct value. This tool **will** be modeled after gofmt's args:

```
-d  Do not print reformatted sources to standard output.    
    If a file's formatting is different than artfmt's, print diffs 
    to standard output.

-l  Do not print reformatted sources to standard output.
    If a file's formatting is different from artfmt's, print its name   
    to standard output.

-w  Do not print reformatted sources to standard output.    
    If a file's formatting is different from artfmt's, overwrite it with 
    artfmt's version.

```

The command includes the spec for *how* artifacts should be formatted,
and is essential for the operation of [[SPC-web]], since it is the same
format that artifacts will be saved in when they are edited.

Several items are essential for easy viewing of formatted artifacts and
avoiding cluttered artifact files:
- For the `partof` field, all calculated items (i.e. REQ-foo-bar is 
  automatically a partof REQ-foo) are removed when storing in text
- Long partof names should be split onto multiple lines
- Any text block that contains newlines are formatted on multiple lines

# SPC-cmd-init
`art init` is the primary first command that will be run by the user when they
want to create a artifact project. It will initialize a `.art` folder in the cwd
(giving an error if the cwd is already initialized) and will give the user basic
instructions on how to create requirements and where to get more information.

# SPC-cmd-tutorial
There **shall** be a tutorial that helps the user learn about artifact.

The tutorial should be interactive and guide the user first through the basics of artifact
and then through recording the requirements and design specifications of a simple
project.

Key points that should be hit during the tutorial are:
- how to use `artifact` and the `ls` cmd
- basics of setting the paths to load docs
- creating requirements
- creating specifications
- creating tests
- auto-linking of req <- spc <- of similar names
- manual linking of different names
- format for linking multiple items
- debugging when links are invalid
- marking items as done
- marking items as tested
- overview of error messages and error formats
- opening the web-ui and exporting html
- final words

# TST-cmd
tests for the cmdline interface

# TST-cmd-fmt
There shall be extensive testing around the fmt cmd to cover multiple
points of risk:

- the command itself shall "check before writing"
- tests shall apply the changes and then check that data was not lost. 
    If data was lost, it will tell the user
- the command will be frequently used on the requirements for artifact,
    which should help detect failures of the tool

# TST-cmd-init
There shall be a unit test to test basic commands:
- init: just validate that it fails when already initialized and correctly 
    initializes otherwise

# TST-cmd-tutorial
- validate that all lines in the tutorial are 80 characters or less.
- validate you can do a run-through
