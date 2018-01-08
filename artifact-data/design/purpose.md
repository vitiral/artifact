# REQ-data
This defines the "artifact-data" module, a self contained programming API for
deserializing, processing and reserializing artifacts from either strings or a
set of paths to files.

> This is a work in progress

This document intends to give a highly detailed design of
the data module with the goals of:
- simplicity: it should be easy to follow the type structure and function
  logic.
- robustness: all methods should have well defined inputs and outputs. In
  addition, rigourous fuzz testing should be designed in from the very
  beginning.
- speed: many of the slowest operations should now be done concurrently,
  such as file system IO.
- memory usage: reference counts should be used to conserve memory+runtime
  where possible.
- self contained: this module should not depend on any other artifact modules
- testability: a test framework should be built to make it easy to add
  "use case" tests. This test framework should be extendable by other higher
  level test frameworks (i.e. a CLI framework or a web-framework).

This requirement is split into the following
- [[REQ-data-type]]: the valid types of artifacts and the attrs in an artifact.
- [[REQ-data-family]]: the valid relationships between artifacts

# REQ-data-test
This requirement details the *exported* functions, classes and framework for
making testing artifact projects simpler and more robust.

> This does **not** detail the tests for the `artifact-data` crate, although
> those tests do leverage this module.
>
> For the definition of artifact-data tests see [[TST-data]].

There are three pieces of this requirement:
- Definition of exported "helper" methods and types for testing  artifact.
  This is not defined further, but should be clear from reading the test source
  code documents.
- Definition of exported "fuzzing" methods for fuzz testing artifact
- Definition of exported "test framework" for creating examples and assertions
  using a simple file structure.

# SPC-data
The control flow and high level architecture for deserializing and processing
artifact data are defined below. The types are defined in [[SPC-data-structs]].

```dot
digraph G {
    node [shape=box];

    subgraph cluster_start {
        {start [label="paths to parse"; shape=oval ]}
     }
    subgraph cluster_src {
        label=<<b>parse src code links</b>>;
        start -> [[dot:SPC-data-src]];
    }
    subgraph cluster_artifacts {
        label=<<b>parse artifacts</b>>;
        start -> [[dot:SPC-data-raw]]
            -> [[dot:SPC-data-family]];
        "SPC-DATA-RAW";
    }


    // join main and branch
    "SPC-DATA-SRC" -> [[dot:SPC-data-artifact]];
    "SPC-DATA-FAMILY" -> "SPC-DATA-ARTIFACT"
      -> [[dot:SPC-data-lint]]
      -> {done [shape=oval]};
}
```

The following are major design choices:
- **join-data**: combine the data from the indenpendent (parallizable) streams.
- [[SPC-data-cache]]: the "global" caching architecture.
- [[TST-data]]: the overall testing architecture

There are the following subparts, which are also linked in the graph above:
- [[SPC-data-src]]: "deserialize" the source code and extract the links to
  artifacts
- [[SPC-data-raw]]: deserialize the artifact files into "raw" data.
- [[SPC-data-name]]: deserialize the artifact names into objects.
- [[SPC-data-family]]: Determine the family of the artifats.
- [[SPC-data-artifact]]: join the data and calculate the remaining pieces of
  the artifact.

In addition:
- [[SPC-data-lint]]: lints that are done against the artifact data.

# REQ-data-lint
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

# TST-data
Testing the data deserialization and processing, as well as reserialization is a major
concern. The `data` API is used for:
- Loading artifacts at init time.
- Formatting artifacts and dumping them to files (toml, markdown, etc)
- Editing artifacts through the web-ui and revalidating them before dumping them.
- Exporting the artifact as JSON, both for the web-ui and for external tools.

The primary approaches to testing shall be:
- Sanity tests: every data type will have ultra simple human written
  "sanity" tests to verify that they work according to user input.
- [[TST-data-fuzz]]: scaleable fuzz testing design
- [[TST-data-interop]]: interop testing strategy.

# TST-data-framework
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

# TST-data-fuzz
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
