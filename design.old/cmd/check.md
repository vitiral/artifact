# SPC-cmd-check
The `check` command gives the user information on any errors that exist in
the project artifacts such as:
 - errors during loading
 - invalid artifacts in partof
 - extra "locations" provided in the code
 - "hanging" non-REQ artifacts that are not partof any other artifacts.

Note: "check" was chosen over "status" because it does NOT mimick the `git status`
command, which doesn't return errors (and isn't meant as a check). It is more
like a compile checker or linter -- in which "check" makes more sense.

This command is intended to be used along with automated testing before PR, etc are
merged into a stable branch.

# TST-cmd-check
simply load up an environment that displays all the errors and validate that
they are displayed in a reasonable way.
