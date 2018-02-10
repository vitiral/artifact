# SPC-cmd-ls
The `art ls` command shall be used to list information the artifacts in a
project.

`ls` is the primary window into a user's artifacts, creating a simple
interface to glimpse large amounts of information.

## Arguments
`art ls` with no commands will simply print all artifacts with standard
settings, each on a single line.

The args are as follows:
- SEARCH str: positional argument detailing which artifacts to display
  By default, this will be interpreted as an Artifact Name and can therefore
  only display one artifact. However, if pattern searching is selected, it 
  will be interpreted as a rust regexp
- display: flags that control what information will be displayed
- pattern: searh SEARCH with a regex pattern. The flag specifies which fields
  should be searched.
- completed/tested: flags which control what percentage completed/tested to
  display

Additionaly, the following is defined:
- [[SPC-cmd-ls-color]]: how the output is colorized
- [[SPC-cmd-ls-display]]: display options (long, short, etc)
- [[SPC-cmd-ls-pattern]]: more details on the pattern option.

# SPC-cmd-ls-color
In order to make viewing of information via cmdline easier,
artifact **shall** colorize it's output to make it clear which items
are done or not done or in error.

The following **will** be followed:
- names that are complete will be green
- names that are almost complete will be blue
- names that are somewhat complete will be yellow
- names that are very litle or not complete will be red
- names that are in ERROR will *blink bold red*

For completed, the levels are: (100%, 70%, 40%, 0%) which correspond to points
(3, 2, 1, 0) and colors (blue, yellow, yellow, red)

For tested, the levels are: (100%, 50%, 0%) which correspond to points (2, 1, 0)
and colors (blue, yellow, red)

Add these together and you get the following:
- 5: Everything green
- 3-4: name is blue
- 1-2: name is yellow
- 0: name is red

# SPC-cmd-ls-display
The following flags are used to specify what to display:
- a/A: display all these flags for all artifacts
- D: display the path to where the artifact is defined
- P: display parts names in reduced form
- O: display partof names in reduced form
- T: display the text formatted as markdown. If `-l` is not specified, this
  will display up to 50 chars of the first line of the text, truncating it
  with ... if necessary.
- L: display the loc path (implementation path)

# SPC-cmd-ls-pattern
the `-p` flag will signify that the SEARCH argument should be interpreted as
a regexp pattern instead of as artifact names

If a value follows p, it will specify the fields to filter in that map with
REQ-ls-display (with the addition of `N` for name)

So: `art ls -p "REQ-ui-cmdline.*" -NO` would filter by name and partof

# TST-cmd-ls
partof:
- SPC-cmd-ls-color
- SPC-cmd-ls-display
- SPC-cmd-ls-pattern
###
Tests for ls are relatively straightforward and mostly involve:
- having an example project
- running the ls command with various different parameters
- validating the output looks as expected
- checking that long and multi-line text is split up as it should be
