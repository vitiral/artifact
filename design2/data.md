# REQ-data
- partof: REQ-1
###
This defines the "data" module, a self contained programming API
for deserializing, processing and reserializing artifacts from
either strings or a set of paths to files.

> This module is being completely redesigned since 1.0 using
> graphiz flowcharts as well as new features like artifacts
> written in markdown. This is a work in progress!

This document intends to give a highly detailed design of
the data module with the goals of:
- simplicity: the types should not overlap in purpose
- robustness: all methods should have well defined inputs and outputs. In
  addition, rigourous fuzz testing should be designed in from the very
  beginning.
- speed: many of the slowest operations should now be done concurrently,
  such as file system IO.
- memory usage: reference counts should be used to conserve memory+runtime
  where possible.
- self contained: this module should not depend on any other.

# SPC-data
The control flow and high level architecture for deserializing and processing
artifact data are as follows:

```dot.svg
digraph G {
    {start [label="paths to parse"]}

    // Links path
    start -> par_deser -> {
        deser_src [
            label="Deserialize source links\n->Map<Name, CodeLocs>";
            href="[[@SPC-data-src]]";
            shape=box;
        ]
    };

    // Artifacts path
    start -> par_art -> {
        deser_art [
            label="Deserialize artifacts\n->Map<Name, RawArtifact>";
            href="[[@.raw]]";
            shape=box;
        ]
    };

    deser_art -> {
        deser_names [
            label="Deserialize names into objects";
            href="[[@.names]]";
            shape=box;
        ]
    };

    deser_names -> par_lint_links -> {
        lint_links [
            label="Lint broken text links";
            href="[[@SPC-data-lint-links]]";
            shape=box;
        ]
    };

    deser_names -> {
        auto_partof [
            label="Determine auto-partofs";
            href="[[@SPC-data-auto_partof]]";
            shape=box;
        ]
    };

    // join main and branch
    {
        join_main [
            label="join main links";
            href="[[@SPC-data-parallel]]";
            shape=box;
        ]
    };
    deser_src -> join_main;
    deser_art -> join_main;

    // after main join
    join_main -> par_lint_subnames -> {
        lint_subnames [
            label="lint subnames";
            href="[[@SPC-data-lint-subnames]]";
            shape=box;
        ]
    };

    join_main -> par_lint_src_links -> {
        lint_src_links [
            label="lint src links";
            href="[[@SPC-data-lint-src]]";
            shape=box;
        ]
    };

    join_main -> par_compl -> {
        compl [
            label="Determine completeness";
            href="[[@SPC-data-completeness]]";
            shape=box;
        ]
    }

    compl -> {
        combine [
            label="Combine to create Artifacts";
            href="[[@.combine]]";
            shape=box;
        ]
    };

    compl -> done
}
```

The following are major design choices:
- [[SPC-data-parallel]]: the general parallization architecture.
- [[SPC-data-cache]]: the "global" caching architecture.
- [[TST-data]]: the overall testing architecture

There are the following subparts, which are also linked in the graph above:
- [[SPC-data-src]]: "deserialize" the source code and extract the links to
  artifacts
- [[.raw]]: deserialize the artifact files into "raw" data.
- [[.names]]: deserialize the artifact names into objects.
- [[SPC-data-auto_partof]]: Determine the auto-partofs into a Map.
- [[SPC-data-completeness]]: Calculate implemented% and tested%.
- [[.combine]]: combine to create artifacts.

In addition:
- [[SPC-data-lint]]: specified lints
- [[SPC-data-ser]]: serialization specification

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

TODO: this needs to be flushed out with the specific libraries to use and how to use them.

# TST-data-interop
There shall be targeted interop testing for *specific* risks. I want to get
away from broad interop tests for this module (they can still exist for the
larger application).

Interop testing should use fuzz testing as well to auto-construct projects. This is
especially true for high risk features like ser/deser where data could
theoretically be lost for inputs that a human might not expect.
