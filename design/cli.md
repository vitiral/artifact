# SPC-cli
partof: REQ-purpose
###

The CLI is the primary developer interatction with artifact, alongside the Web
UI for visualizing the artifacts. The main goal of the CLI is to provide the
tools that every developer is used to in a typical development tool. This
includes:


- [[.init]]: Initialize a project for using artifact. This is pretty basic, just need
  a `.art` folder with a `settings.toml` and an initial `design/` folder.
- [[.check]]: checking for errors AND warnings with a return code if there is an error.
    - If there are only warnings the return code == 2. Otherwise it == 1.
- [[.fmt]]: auto format the project.
  - `--type` flag to change the filetype.
- [[SPC-cli-ls]]: listing/searching for artifacts, see the full specification.


All subcommands should include the following flags:
- `-v / --verbose` for setting the logging verbosity.
- `--work-dir` for setting the working directory to run the command out of.


# SPC-cli-ls
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
- [[.long]]: the `-l` flag prints the artifact in "long" form. Without it it
  is printed in [[.table]] form.

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
