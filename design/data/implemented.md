# SPC-impl
partof: REQ-data
###
Implementing artifacts is fairly straight forward:
- [[.done]]: the artifact can define itself as done. If it does this, it must
  not have any subnames and must not be implemented (linked) in source.
- [[.subnames]]: subnames can be defined in the `text` field of the artifact
  (see the beginning of this line for an example!)
- [[SPC-read-impl]]: source code can link to either the artifact itself or one
  of its subnames through `#ART-name` or `#ART-name.sub` respectively.
-


# SPC-read-impl
partof: SPC-impl
###
## Loading source code (implementation) links

### [[.load]]: Loading Locations
The process for loading implementation locations is fairly straightforward:
- Define the regular expression of valid names. Valid names inclue:
  - `SRC` and `TST` types ONLY.
  - Any valid postfix name (i.e. `SPC-foo-bar-baz_bob`)
  - (optional) a sub-name specified by a period (i.e. `SPC-foo.sub_impl`).
- Walk the `code_paths`, iterating over each line for the regex and pulling
  out any `Name` or `SubName` locations.

This results in two maps for each file:
- `Name => CodeLoc`
- `SubName => CodeLoc`

## Lints
All lints related to source code are only WARNINGS

- [[.lint_done]]: an artifact with its `done` field set is also linked
  in code.
- [[.lint_exists]]: the artifact name does not exists but it does not specify the
  linked
- [[.lint_subname_exists]]: the artifact name exists but the artifact does not specify
  the linked subname.

### [[.join]]: Joining Locations
The `Name` and `SubName` maps from each file are joined into two large maps
respectively (with any collisions put in the linting vectors which are also
joined).

We must then construct a map of `Name => Implementation` in order for later
steps to construct the full `Artifact` object. We do this by:
- Constructing a map of `Name => Map<SubName, CodeLoc>`, where `Name` is the
  prefix/name of the underlying `SubName`s.
- Building the `Name => Implementation` map by:
  - Draining the `Name => CodeLoc` map and inserting `Implementation` objects.
  - Draining the just created `Name => Map<SubName, CodeLoc>` and either
    modifying or inserting `Implementation` objects.

> Note: we do not worry about whether such `Name` or `SubName`s actually exist.
> That is the job of a later linting step.