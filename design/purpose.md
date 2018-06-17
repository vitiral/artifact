# REQ-purpose
> This is a work in progress
>
> These are the design documents. For user documentation
> see the project's [README][artifact].

**These are the developer design documents. For user documents and project
information, see the project's [README][artifact].**

## Overview for Artifact
The goal of artifact is to be a simple, linkable and trackable design
documentation tool for everybody.

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
- Be scalable to any size of project (i.e. fast+cached).

These features will empower developers to track their own design docs and make
it possible for them to use their design docs to provide documentation and
guidance for contributors and teamates.

[artifact]: https://github.com/vitiral/artifact

## What is an "Artifact"?
`artifact` allows the user to define any number of what it calls "Artifacts".
An Artifact is simply a part of a document (i.e. json, toml or extended
markdown) which:
- Is a single entity composed of a specific set of attributes defined below.
- [[SPC-name]]: Has a project-wide unique `Name` beginning with one of `REQ`,
  `SPC` or `TST`, which is the artifact's `Type` ([[SPC-name.type]]).
- [[SPC-family]]: Has a `partof` attribute which makes it a dependency of other
  artifacts.
- [[SPC-impl]]: Has a `text` attribute which allows you to write out the
  artifact's specification, as well as create subnames which can be linked in
  source code (i.e. `ART-name.subname`)
  - [[SPC-impl.done]]: alternatively can be forced as implemented through the
    `done` attribute
- [[SPC-read-impl]]: Can be linked in source code using `#ART-name` or
  `#ART-name.subname`, which allows you to "implement" the artifact directly.
- [[SPC-read-artifact.completed]]: Tracks spc and tst completion of
  artifacts by (roughly) averaging the completeness of their children + the
  completion of their subparts, where TST's only affect the TST completion of
  REQ and SPC.

Artifacts are first and formost intended to be simple and lightweight. They
try to stay out of your way and *express what you want, when you want it*.

## Design Architecture
The design of artifact is split into several sub-modules

- [[REQ-data]]: the "data" module, which acts as a filesystem database for CRUD
  operations on the user's artifacts.
- [[SPC-cli]]: The CLI interface. Artifact always aims to be a "developer first" tool, and
  having a full featured CLI with search+lint+export commands is one of the ways it
  accomplishes that goal.
- [[REQ-web]]: the webui frontend/backend implementation, which is one of the
  main ways that users actually use artifact.


# REQ-web
partof: REQ-purpose
###
The web interface for artifact should be designed to behave very similar to the
CLI/text based interface, except it should take advantage of everything that a web
interface can.

Main attributes:
- Works directly on the file system. Any database introduced should only be used for improving performance (invisible to the user).
- [[.secure]]: this is definitely TODO, not sure how it will be accomplished ATM. Probably just require non-local host hosting to require a password and use HTTPS (nothing crazy).
- Fast for _single users and small groups_. Explicitly not designed as a whole org editing portal, users are encouraged to make small changes and use existing code review tools for changing design docs.

# Architecture
The basic architecture of the web UI is split into two components:
- [[.backend]]: this will be a simple json-rpc server which uses the [[REQ-data]] crate
  to do all of it's heavy lifting. [[REQ-data]] will ensure data consistency and error
  handling.
- [[.frontend]]: the frontend will be a single page application which
  accomplishes a majority of the goals of artifact, including real-time
  feedback, graphing and visualization of requirements. It and the CLI are the
  two major "user facing" components of artifact.


# TST-fuzz
partof:
- SPC-modify
- SPC-modify-update
- SPC-name
- SPC-read
- SPC-read-artifact
- SPC-read-family
- SPC-read-impl
###

All data objects shall be designed from the beginning to be fuzz tested, so
that even complex "projects" can be built up with random collections of
artifacts in differing states.

Obviously this will also allow for fast fuzz testing of the smaller objects themselves.

The major workhorse here will be the [quickcheck][1] library. The following datatypes
will have `Abitrary` implemented for them and releated tests performed against them:
- `Name` (and by extension `Partof`)
- `InvalidName`
- `RawArtifact`
  - `Done`
  - `CodeRef`
  - `CodeLoc`
  - `Text`
- `RawCodeLoc`: simply a file with given code references inserted at random.
- `HashMap<Name, RawArtifact>`
- etc.

Fuzz testing will then involve the following:
- positive fuzz tests: it should handle all generated cases that are expected
  to work.
- negative fuzz tests: it should handle all generated cacses that are expected
  to fail properly.

[1]: https://docs.rs/quickcheck/0.4.2/quickcheck/

# Implementations
- [[.raw_name]]
- [[.name]]
- [[.family]]
- [[.read_impl]]
- [[.artifact]]
- [[.read]]
- [[.modify]]
- [[.modify_update]]


# TST-unit
partof:
- SPC-modify
- SPC-modify-update
- SPC-name
- SPC-read-artifact
- SPC-read-family
- SPC-read-impl
###

Several low level specifications can be covered almost completely with unit
and fuzz testing ([[TST-fuzz]]) testing.

In order for an item to consider itself "unit tested" it must:
- test boundary conditions
- test error conditions
- test "sanity" use cases (standard use cases)


# Implementations
- [[.raw_name]]
- [[.name]]
- [[.family]]: this also inclused auto partofs as well as collapsing/expanding
  partof.
- [[.read_impl]]
- [[.artifact]]
- [[.modify]]
- [[.modify_update]]