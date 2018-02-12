# REQ-purpose
The goal of artifact is to be a simple, linkable and trackable design
documentation tool for everybody.

These are the design documents. For the overarching purpose,
see the project's [README][artifact].

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
- Be scalable to any size of project (i.e. fast).

These features will empower developers to track their own design docs and make
it possible for them to use their design docs to provide documentation and
guidance for contributors and teamates.

[artifact]: https://github.com/vitiral/artifact
