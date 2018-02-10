# REQ-purpose
The goal of artifact is to be a simple, linkable and trackable design
documentation tool for everybody.

These are the design documents. For the overarching purpose,
see the project's [README][artifact].

This may seem trivial, but it's not. A useful design doc tool must have *at least*
the following characteristics:
- Allow simple linking of requirements -> specifications -> tests.
- Easily link to source code (through the source documentation) to determine
  completeness.
- Be revision controllable (text based).
- Have a unix-like command line interface for interacting with your design docs.
- Have a web-ui for viewing and editing rendered documents.
- Provide interop functionality like subcommand and data export for integration
  with external tools and plugins.
- Be scalable to any size of project (i.e. fast).

These features will empower developers to track their own design docs and make
it possible for them to use their design docs to provide documentation and
guidance for contributors and teamates.

[artifact]: https://github.com/vitiral/artifact


# SPC-ls
The `art ls` command shall be used to list information about the artifacts in a
project.

`ls` is the primary window into a user's artifacts, creating a simple interface
to glimpse large amounts of information.

## [[.args]]: Arguments
`art ls` with no commands will simply print all artifacts with standard
settings, each on a single line.

The args are as follows:
- `SEARCH str`: positional argument detailing which artifacts to display
  By default, this will be interpreted as an Artifact Name and can therefore
  only display one artifact. However, if pattern searching is selected, it
  will be interpreted as a rust regexp
- `display`: flags that control what information will be displayed
- `pattern`: searh SEARCH with a regex pattern. The flag specifies which fields
  should be searched.
- `completed/tested`: flags which control what percentage completed/tested to
  display

Additionaly, the following is defined:
- [[SPC-cmd-ls-color]]: how the output is colorized
- [[SPC-cmd-ls-display]]: display options (long, short, etc)
- [[SPC-cmd-ls-pattern]]: more details on the pattern option.

## [[.color]]: Color
In order to make viewing of information via cmdline easier, artifact **shall**
colorize it's output to make it clear which items are done or not done or in
error.

The following are the general rules:
- Names that are complete will be `green`.
- Names that are almost complete will be `blue`.
- Names that are somewhat complete will be `yellow`.
- Names that are very litle or not complete will be `red`.
- Names that are in ERROR will be `bold red`.

For [[.color_spc]], the levels are:
- `( 100%,  70%,    40%,  0%)`: percentage spc
- `(    3,    2,      1,   0)`: points
- `(green, blue, yellow, red)`: colors

For [[.color_tst]], the levels are:
- `( 100%,    50%,  0%)`: percentage tst
- `(    2,      1,   0)`: points
- `(green, yellow, red)`: colors for tst

For [[.color_name]] you add the two points together:
- 5: Name is Green
- 3-4: name is blue
- 1-2: name is yellow
- 0: name is red
