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

This requirement is split into the following
- [[REQ-data-type]]: the valid types of artifacts and the attrs in an artifact.
- [[REQ-data-family]]: the valid relationships between artifacts

# SPC-data
The control flow and high level architecture for deserializing and processing
artifact data are defined below. The types are defined in [[SPC-data-structs]].

```dot
digraph G {
    splines=ortho;
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
        "SPC-DATA-RAW" -> [[dot:SPC-data-lint-text]];
    }


    subgraph cluster_join {
        label=<<b>join and process</b>>;
        {join [label="join data"]};
        [[dot:SPC-data-completeness]];
        "[[dot:.combine]]";
    }

    // join main and branch
    "SPC-DATA-SRC" -> join;
    "SPC-DATA-FAMILY" -> join
        -> "SPC-DATA-COMPLETENESS"
        -> "[[dot:.combine]]" -> {done [shape=oval]};


    join -> [[dot:SPC-data-lint-src]]
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
- [[SPC-data-completeness]]: Calculate implemented% and tested%.
- [[.combine]]: combine to create artifacts.

In addition:
- [[SPC-data-lint]]: specified lints

# SPC-data-completeness
TODO

# SPC-data-lint
## Lint Design

> The design of how linting will be handled is very important to the simplicity
> of the data flow. Often times "warning" and "non-fatal" level errors are
> overlooked in the initial design, even put to the job of global logging
> handlers. It is intended that that is avoided here.
>
> Note: In cases where an error is "critical" it will not be a lint, it will be
> in the `Result::Err` type.

The basic design of lints is that each function that *can* do a full lint with
the information it has *will* do a full lint, and will return it's lint
information as a Set of `Lint` objects.

In cases where functions do NOT have complete information, linting will be left
to later functions.

The `Lint` type is:
```
enum Lint {
    category: LintCategory,
    path: Option<PathAbs>,
    line: Option<u64>,
    msg: LintMsg,
}

#[derive(Hash)]
enum LintCategory {
    ParseCodeImplementations,
    ParseArtifactFiles,
    ... etc
}

enum LintMsg {
    Error(String),
    Warn(String),
}
```

The intention is that `Lint::Error` will cause an application built on artifact
to *not continue* to any final steps where as `Lint::Warn` will only be printed.

When printing lints (at the application level) they should be sorted and
grouped by their categories+files. Each lint should be printed on their own
line.

# SPC-data-lint-src
This is pretty basic: it is a warning to have dangling references in your
source code.

The names and subnames obtained from the source code must be checked against
the defined names+subnames. If a reference exists in source that is not defined
it is a warning.

# SPC-data-lint-text
> This is *not* for linting references in text. That is done at a later step.

There are a couple of invalid items in text that need to be linted.

- `^#\sART-name$`: in the markdown format these get interpreted as individual artifacts.
- `^###$`: in the markdown format these get interepreted as "end of data" lines.

# SPC-data-src
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

Along with two linting vectors for any collisions.

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

# TST-data-interop
There shall be a "interop test harness" constructed for doing interop testing.
The basic design is:
- *Each test is a full project* (folder with a `.art` folder, source files and
  design documents).
- Each test contains assertions at `path/to/test/assert.yaml`
- The assertions file structure is:
  - error: regular expression of the expected error (invalidates other assertions)
  - lints:
    - contains: list of lints that should exist
    - not_contains: list of lints that should not exist
    - length: optional total length
    - Notes:
      Each lint has the fields specified by the struct, with "NONE" being reserved
      for specifying that the value should ACTUALLY be `None` (rather than
      just not specified)
  - visited_artifact_paths:
    - contains: list of visited paths
    - not_contains: list of unvisited paths
    - length: optional total length
  - visited_source_paths:
    - contains: list of visited paths
    - not_contains: list of unvisited paths
    - length: optional total length
  - artifacts:
    - contains: list of artifacts which should exist and the expected value of
      their fields.
    - not_contains: list of artifact names which should not exist.
    - length: optional total length
    - Notes:
      Similar to lints, attributes which are not specified are not asserted but
      NONE is special.
- The test harness then loads the project and assertions file and asserts all
  of the assertions.
