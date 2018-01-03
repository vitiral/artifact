# SPC-data-artifact
The artifact type itself must be constructed from its joined parts.

The design up until this point has been to create mappings of
`Name => piece`, where `piece` is something needed to construct
the artifact. That design will continue here to allow for reduced
complexity and easier testability (where needed).



# TST-data-artifact
Although the different pieces are separated out, most of the "construction" of
the artifact objects themselves will not be tested explicitly. Instead
we will rely on the framework to quickly test user scenarios and the already
existing fuzz framework for more end-to-end tests.

Sanity tests can be added as-needed for each component.

The following sanity tests shall exist:
- [[.completed]]: sanity test computing the `Completed` fields.
