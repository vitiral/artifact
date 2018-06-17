# REQ-data
partof: REQ-purpose
###

The `artifact-data` crate defines a lightweight and robust API for
deterministically deserializing, processing and reserializing artifacts from a
project directory.

This crate treats an "artifact project" (which is a path to a folder
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


# SPC-frontend
partof: REQ-web
###
The frontend for artifact is probably the most important piece of design for creating
a pleasant user experience.

The frontend shall be presented as a single page application with the following
major components:
- [[.view]]: the user should be able to view an artifact and its relationship to other artifacts.
- [[.edit]]: the user should be able to create, modify and delete artifacts.
- [[.search]]: the user should be able to search artifacts and easily see how all artifacts
  relate to each other.


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
artifact data are defined below. The types are defined in [[SPC-structs]].

```dot
digraph G {
    node [shape=box];

    subgraph cluster_start {
        {start [label="paths to parse"; shape=oval ]}
     }
    subgraph cluster_src {
        label=<<b>parse src code links</b>>;
        start -> [[SPC-read-impl]];
    }
    subgraph cluster_artifacts {
        label=<<b>parse artifacts</b>>;
        start -> [[SPC-read-raw]]
            -> [[SPC-read-family]];
        "SPC-DATA-RAW";
    }


    // join main and branch
    "SPC-DATA-SRC" -> [[SPC-read-artifact]];
    "SPC-DATA-FAMILY" -> "SPC-DATA-ARTIFACT"
      -> [[SPC-lint]]
      -> {done [shape=oval]};
}
```

The following are major design choices:
- **join-data**: combine the data from the indenpendent (parallizable) streams.
- [[SPC-test]]: the overall testing architecture

There are the following subparts, which are also linked in the graph above:
- [[SPC-read-impl]]: "deserialize" the source code and extract the links to
  artifacts
- [[SPC-read-raw]]: deserialize the artifact files into "raw" data.
- [[SPC-name]]: deserialize the artifact names into objects.
- [[SPC-read-family]]: Determine the family of the artifats.
- [[SPC-read-artifact]]: join the data and calculate the remaining pieces of
  the artifact.


# SPC-test
partof: REQ-data
###
This requirement details the *exported* functions, classes and framework for
making testing artifact projects simpler and more robust.

There are three pieces of this requirement:
- Definition of exported "helper" methods and types for testing  artifact.
  This is not defined further, but should be clear from reading the test source
  code documents.
- Definition of exported "fuzzing" methods for fuzz testing artifact
- Definition of exported "test framework" for creating examples and assertions
  using a simple file structure.


# TST-framework
partof: REQ-purpose
###
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