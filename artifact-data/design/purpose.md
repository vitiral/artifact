# REQ-data
> This is a work in progress

**These are the developer design documents. For user documents and project
information, see: https://github.com/vitiral/artifact**

## Overview for Artifact
The purpose of artifact is to provide a simple design documentation tool
for developers.

This may seem trivial, but it's not. A useful design doc tool must have *at least*
the following characteristics:
- Allow simple linking of requirements -> specifications -> tests.
- Easily link to source code (through the source documentation) to determine
  completeness.
- Be revision controllable (text based).
- Have a unix-like command line interface for interacting with your design
  docs.
- Have a web-ui for viewing and editing rendered documents.
- Provide interop functionality like subcommand and data export for integration
  with external tools and plugins.
- Be scalable to any size of project.

These features will empower developers to track their own design docs and make
it possible for them to use their design docs to provide documentation and
guidance for contributors and teamates.

## What is an "Artifact"?
`artifact-data` allows the user to define any number of what it calls
"Artifacts". An Artifact is simply a part of a document (i.e. json, toml
or extended markdown) which:
- Is a single entity composed of a specific set of attributes defined below.
- [[SPC-name]]: Has a project-wide unique `Name` beginning with one of `REQ`,
  `SPC` or `TST`, which is the artifact's `Type` ([[SPC-name.type]]).
- [[SPC-partof]]: Has a `partof` attribute which makes it a dependency of other
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

## Overview For This Crate
This crate defines "artifact-data", a lightweight and robust API for
deterministically deserializing, processing and reserializing artifacts from a
project directory.

This module treats an "artifact project" (which is a path to a folder
containing a `.art/` directory) as a transactional database, ensuring
consistency and validity when any update is made. It does this by creating a
unique-hash of the reduced form of any artifact, and requires all updates to
that database to have the *correct* original unique-hash before updates are
allowed.

This allows external applications to use the filesystem *itself* as a "database"
of sorts, and allows you to simultaniously edit artifacts via (for example)
*both* a Web-UI *and* a local text-editor and see real-time updates in both
places -- without having to worry about loosing any of your work.

These requirements are implemented through the following specifications:
- [[SPC-read]]: the "read" part of the CRUD database. Allows you to load an
  artifact project.
- [[SPC-modify]]: the "Create + Update + Delete" part of the CRUD database.
- [[SPC-structs]]: the exported types of this module and their purpose.
- [[SPC-name]]: the valid types of artifacts and the attrs in their name.
- [[SPC-family]]: the valid and automatic relationships between artifacts
- [[SPC-impl]]: how artifacts are implemented.
- [[SPC-lint]]: the design of error handling (spoiler: it's all "lints")

The following test helpers are exported under feature flag `test-helpers`:
- [[TST-fuzz]]: this library shall export **and use** fuzz testing primitives
  throughout its infrastructure
- [[TST-framework]]: this library shall export **and use** a testing framework
  which allows you to express the expected state of a project _as only data_
  and which makes clear assertions using these expected values.

# SPC-lint
partof: REQ-data
###
## Lint Design

> The design of how linting will be handled is very important to the simplicity
> of the data flow. Often times "warning" and "non-fatal" level errors are
> overlooked in the initial design, even put to the job of global logging
> handlers. It is intended that that is avoided here.

The basic design of lints is that:
- Every "error", no matter how severe, should always be cast into a lint.  We
  load lots of files, it is better to simply list all errors rather than fail
  at each one individually.
- Loading lints should be errors and the calling functions are *required* to
  not proceed if there are load errors.
- Other lints should always be *repeatable*, meaning you can rerun the lints
  or even run lints on a project passed by some other means (i.e. from a
  json-rpc call).

## Basic Design

The `Lint` type is:
```
enum Lint {
    level: Level,
    category: Category,
    path: Option<PathBuf>,
    line: Option<u64>,
    msg: String,
}

#[derive(Hash)]
enum Category {
    ParseCodeImplementations,
    ParseArtifactFiles,
    ... etc
}

enum LintMsg {
    Error(String),
    Warn(String),
}
```

The intention is that `Level::Error` will cause an application built on artifact
to *not continue* to any final steps where as `Lint::Warn` will only be printed.

When printing lints (at the application level) they should be sorted and
grouped by their categories+files. Each lint should be printed on their own
line.

# SPC-read
partof: REQ-data
###
The control flow and high level architecture for deserializing and processing
artifact data are defined below. The types are defined in [[SPC-read-structs]].

```dot
digraph G {
    node [shape=box];

    subgraph cluster_start {
        {start [label="paths to parse"; shape=oval ]}
     }
    subgraph cluster_src {
        label=<<b>parse src code links</b>>;
        start -> [[dot:SPC-read-impl]];
    }
    subgraph cluster_artifacts {
        label=<<b>parse artifacts</b>>;
        start -> [[dot:SPC-read-raw]]
            -> [[dot:SPC-read-family]];
        "SPC-DATA-RAW";
    }


    // join main and branch
    "SPC-DATA-SRC" -> [[dot:SPC-read-artifact]];
    "SPC-DATA-FAMILY" -> "SPC-DATA-ARTIFACT"
      -> [[dot:SPC-read-lint]]
      -> {done [shape=oval]};
}
```

The following are major design choices:
- **join-data**: combine the data from the indenpendent (parallizable) streams.
- [[TST-read]]: the overall testing architecture

There are the following subparts, which are also linked in the graph above:
- [[SPC-read-impl]]: "deserialize" the source code and extract the links to
  artifacts
- [[SPC-read-raw]]: deserialize the artifact files into "raw" data.
- [[SPC-read-name]]: deserialize the artifact names into objects.
- [[SPC-read-family]]: Determine the family of the artifats.
- [[SPC-read-artifact]]: join the data and calculate the remaining pieces of
  the artifact.

# SPC-test
partof: REQ-data
###
This requirement details the *exported* functions, classes and framework for
making testing artifact projects simpler and more robust.

> This does **not** detail the tests for the `artifact-data` crate, although
> those tests do leverage this module.
>
> For the definition of artifact-data tests see [[TST-read]].

There are three pieces of this requirement:
- Definition of exported "helper" methods and types for testing  artifact.
  This is not defined further, but should be clear from reading the test source
  code documents.
- Definition of exported "fuzzing" methods for fuzz testing artifact
- Definition of exported "test framework" for creating examples and assertions
  using a simple file structure.

# TST-framework
> TODO: make it partof REQ-data
> partof: REQ-data
> ###

There shall be a "interop test framework" constructed for doing interop testing.
The basic design is:
- *Each test is a full project* (folder with a `.art` folder, source files and
  design documents).
- Each test contains assertions in various files. The assertions cover various
  stages of loading the project:
  - `project/assert_load_lints.yaml`: expected lints while loading.
  - `project/assert_project.yaml`: the expected resulting project. If not included,
    it is expected that the project is `None`.
  - `project/assert_project_lints.yaml`: expected lints from linting the project.
- The assertion files are an exact *explicit* version of the expected project.

# TST-fuzz
> TODO: make it partof REQ-data
> partof: REQ-data
> ###

All data objects shall be designed from the beginning to be fuzz tested, so
that even complex "projects" can be built up with random collections of
artifacts in differing states.

Obviously this will also allow for fast fuzz testing of the smaller objects themselves.

The old API used `Type::fake()` in lots of places -- these are good flags for *some* of
the places that fuzz testing could have been used instead (but the examples are much
larger than just that).

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

From the implementations, we can randomize testing for the following:
- [[.load_name]]: use `Name` and `InvalidName` to great effect.
- [[.load_artifacts]]: simply convert randomly generated artifacts into files
- [[.load_src]]: load RawCodeLoc and have expected result

[1]: https://docs.rs/quickcheck/0.4.2/quickcheck/

# TST-read
Testing the data deserialization and processing, as well as reserialization is a major
concern. The `data` API is used for:
- Loading artifacts at init time.
- Formatting artifacts and dumping them to files (toml, markdown, etc)
- Editing artifacts through the web-ui and revalidating them before dumping them.
- Exporting the artifact as JSON, both for the web-ui and for external tools.

The primary approaches to testing shall be:
- Sanity tests: every data type will have ultra simple human written
  "sanity" tests to verify that they work according to user input.
- [[TST-fuzz]]: scaleable fuzz testing design
- [[TST-read-interop]]: interop testing strategy.
