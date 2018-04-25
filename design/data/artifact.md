# SPC-read-artifact
The artifact type itself must be constructed from its joined parts.

The design up until this point has been to create mappings of
`Name => piece`, where `piece` is something needed to construct
the artifact. That design will continue here to allow for reduced
complexity and easier testability (where needed).

# [[.load]]: calculate pieces from ArtifactRaw only
After we have successfully loaded all of the artifacts we still want
to calculate all of the pieces such as `partof`, `parts` and `subnames`.

This is fairly straightforward, but the following should be kept in
mind.
- `subnames` is a simple iterative regexp search of `text`
- `partofs` simply has to concatenate any auto-partof values from the `family`
  module (see [[SPC-read-family.auto]]).
- Create a graph using `parofs` and use it to calculate `parts`. We need the
  graph later anyway.

# [[.build]]: build the artifact from its parts
After we have successfully loaded and finalized the artifact pieces, we need
to combine them with the implementations and calculate completeness.

This mostly just has to make use of the functions defined in

## [[.graph]]: construction of the graph
The construction of the graph takes a map of each artifact to its given+auto
`partof` field, which defines the reverse directed edges of the graph.

We requre three graphs:
- `full`: this is primarily used to compute the `parts` field and to later lint
  that there are no cycles in the graph.
- `tst`: graph containing `(TST, TST)` edges ONLY. Because TST can only be
  a `partof` other TST artifacts, we know that the dependencies of this graph
  is always independently solvable.
- `req_spc`: graph containing non `(TST, TST)` edges.

## [[.completed]]: compute the `Completed` objects.
When computing completeness we are basically trying to solve for dependencies.
A graph can help significantly in this by giving us the topological sort.
The toplological sort guarantees that any item can be calculated if its
items to its right are are calculated.

There are a couple of artifact-specific points to keep in mind:
- TST `spc` completenss is always equal to it's `tst` completeness.
- TST does not contribute towards the `spc` completeness of non-TST types.
- We have to account for the `impl` field on each artifact.

Other than that, we simply:
- Solve for the `graph_tst`, setting `completed.tst = completed.spc` for each
  of its items
- Solve for the `graph_req_spc`, knowing that any `graph_tst` dependencies
  have already been solved.
- If any cycles are detected we just return that all items are 0% completed.
  A later lint will handle that issue.

## Lints
- [[.lint_text]]: ensure that the artifat's text is valid in all of the
  formats. For markdown, this means ensuring that nothing would be parsed
  as "a new artifact" or the "end of metadata" blocks.
- [[.lint_done]]: ensure that done and subnames are not both defined.
- [[.lint_text_refs]]: ensure that soft references (`[[ART-name(.sub)]]`)
  all point to real things.

# TST-read-artifact
Although the different pieces are separated out, most of the "construction" of
the artifact objects themselves will not be tested explicitly. Instead
we will rely on the framework to quickly test user scenarios and the already
existing fuzz framework for more end-to-end tests.

Sanity tests can be added as-needed for each component.

The following sanity tests shall exist:
- [[.partofs]]: sanity test making sure `partofs` are properly determined.
- [[.completed]]: sanity test computing the `Completed` fields.

The major testing will be done using the interop framework. The following
test cases should be implemented:
- [[.empty]]: `empty` project that contains only empty artifact files.
  and no source code.
- [[.design_only]]: a project containing only design documents (none of the
  artifacts implemented).
  - This is mostly to test that artifact parsing works and linking works
  - A few artifacts should be "defined as done" to get some basic "completion"
    testing done as well.
  - Artifacts should be split into lots of deep folders, to push parsing to
    a higher limit.
  - Some artifact folders should be excluded.
- [[.basic]]: a basic project with minimal artifacts, some implemented in
  source code.
  - This is mostly a "sanity" project
  - At least one artifact implemented in source
  - At least one subname implemented in source
  - At least one artifact with subnames NOT implemented in source
  - At least one artifact only partially implemented (no subnames and not implemented)
  - At least one artifact only partially implemented (no primary + single secondary)

- [[.lint]]: a basic project to test lints
  - Expected lint errors:
    - referenes to names+subnames that don't exist
    - partof values that don't exist
    - At least one artifact BOTH implemenented in source and defined as done
    - Invalid reference (name + subane) in source
