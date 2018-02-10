# SPC-project
partof: REQ-artifact
###
The "Project" data type shall encompase the users entire set of
artifact files and settings and the process for loading them.

It includes:
- [[SPC-project-settings]]: specify where to find code and artifacts
- [[SPC-project-load]]: project loading procedure

## Artifact
The artifact type is defined by [[SPC-artifact]]

# SPC-project-load
The project loading procedure will follow the following process:
  1. load settings
  2. follow paths to artifact directories
  3. load artifacts from files
  4. validate name-collisions
  5. validate attribute types
  6. create missing parents
  7. auto-link artifacts
  8. analyze completeness

# Stages

## Stage 1

1. Load a single settings file

## Stage 2

Processes A (in series):
 1. Recursively find all toml files in `artifact_paths` directories
 2. Load all files in parallel into raw Artifact objects

Processes B (in series):
 1. Recursively find all source code files in `code_paths` directories
 2. Parse out implementation locations

## Stage 3:

process the project, see [[SPC-project-process]]

# Error Scenarios
The only locations where critical failure (i.e. failure that causes `ls` to not work)
is possible is in Stage 1 or 2

Critical failure shall occur if:
- fs error
- invalid toml
- artifact errors (invalid name, extra attr, invalid attr type, etc)
- invalid partof string (artifact names)
- invalid artifact_path

Failures in further steps will simply cause warnings and will display on the
ui as missing links or None completion (etc)

# SPC-project-process
partof: SPC-partof
###
Once a project has been loaded (see [[SPC-project-load]]) it has to
be processed. Namely:

- automatic parents defined by [[SPC-partof]] must be created.
- automatic links defined by [[SPC-partof]] must be appended
- partof links must be validated
- the `parts` field defined in [[REQ-artifact]] must be inferred
- `completed` and `tested` as defined in [[SPC-artifact]] must be calculated

This process *must* be idempotent, meaning we must be able to convert
a processed project to data and back multiple times without affecting
the result.

## Stage 1:
Processes A (in series):
 1. create parents
 2. link parents
 3. validate partof
 4. link parts

Processes B (in series):
 1. attach locations

## Stage 2:
 1. set completed and tested

# SPC-project-settings
Project settings shall be definable in a single file located at
{repo}/.art/settings.toml

This file must be in the settings format with the following attributes
The definition of an artifact project shall be due to a file existing at
`.art/settings.toml`. This setings file shall include configuration for
various project-level features such as:

- `artifact_paths`: files or directories to find artifacts
- `exclude_artifact_paths`: artifact_paths to exclude
- `code_paths`: files or directories to find code files that implement SPC or
  TST artifacts
- `exclude_code_paths`: code paths to exclude

[[SPC-project-load]] then uses the information provided to find artifacts.

### Future Developments
In the future a `project_paths` variable may be added with paths to other
projects. It is not yet known whether this featre would be actually useful,
since having all design docs in one place for a large project seems correct,
and in areas where the project can be split they should just have their
own self-contained design documents (which can be referenced by projects
using them using urls).

# TST-project
partof: SPC-artifact
###
These tests represent the unit tests for saving and loading
artifacts to/from files.

These are intended to test, as completely as possible, that
artifacts don't get loaded or processed into invalid states no
matter how many times we handle them.

These are essential for a broad range of services offered including
formatting and saving artifacts in the web-ui.

# TST-project-invalid
partof: SPC-project-load
###
load the following and make sure it results in an error:
 1. trying to input a json-like table `{}`
 2. trying to have multiple types in an array `[1, "hello", 3]`
 3. name collisions at the [] level
 4. name collisions at the base level
 5. `[file]` with an invalid attribute
 6. two files with same key

# TST-project-link
partof:
- SPC-partof
- SPC-project-load
###
design documents linking (both automatically and explicitly) is one of the
core features of artifact and is one that will be expected to "just work"
and be ultra simple.

Unit tests need to focus on multiple failure paths that could crop up
around linking along with basic functionality testing and validate that the
calculations are *exactly* as predicted by manual calculations.

# TST-project-partof
partof: SPC-partof
###
The partof format and loading can be tested completely with simple unit tests:
 - load simple lists
 - load complex lists
 - load varieties of invalid lists

# TST-project-process
A huge amount of functionality requires that we can process a project
that has been converted to its "data" form over and over and that
processing it is completely idempotent. For instance, [[SPC-rpc-artifacts]]
works by creating data artifacts from an existing project, editing those
data artifacts and then re-processing the project to get the new value.

It is critical that converting a project to data and reprocessing it
is a completely idempotent operation.

# TST-project-simple
partof:
- SPC-project-load
- SPC-project-settings
###
Create a simple project that has a few ins and outs
 - has multi-level values
 - has folder that is unreachable unless you set an extra path (which is set)
 - has recursive setting of paths (encouraging double-eval)
 - use default variables extensively

Litter this with artifacts at each level, some of which are implemented in a fake src/ dir
Validate that everything is as it should be
