# REQ-1
These are the developer design documents. For user documents and project information,
see: https://github.com/vitiral/artifact

The purpose of artifact is to provide a simple design documentation tool
for developers.

This may seem trivial, but it's not. A useful design doc tool must have *at least*
the following characteristics:
- allow simple linking of requirements -> specifications -> tests
- easily link to source code (through the source documentation) to determine
  completeness
- be revision controllable (text based)
- have a unix-like command line interface for interacting with your design docs
- have a web-ui for viewing and editing rendered documents
- provide interop functionality like subcommand and data export for integration
    with external tools and plugins
- be scalable to any size of project

These features will empower developers to track their own design docs and make
it possible for them to use their design docs to provide documentation and
guidance for contributors and teamates.

The application requirements are split into the following categories:
- [[SPC-0]]: definitions and pre-design decisions for this project
- [[REQ-artifact]]: details the types of artifact and valid links
- [[REQ-partof]]: details how artifacts are linked by the user
- [[REQ-completion]]: artifact will track completeness
- [[REQ-security]]: security considerations and risks
- [[REQ-cmd]]: details the cmdline interface including `ls`, `check` and `export`
- [[REQ-rpc]]: details the json-rpc interface of the server
- [[REQ-web]]: the web-ui editing interface
- [[REQ-tracker]]: work in progress requirements tracker. May be removed from
  this project (made into an independent project that consumes the artifact json-rpc
  api)

# REQ-artifact
partof: REQ-1
###
artifact **will** support 4 types of artifacts that can be tracked
- **REQ**: software requirement
- **SPC**: software design specificaion based on one or more requirements
- **TST**: test design of a risk or specification

The valid links **will** look like this:
```
  REQ <-- SPC* <-- TST*

* SPC and TST can be implemented in source code
```

In other words, you can design a spec (SPC) based on a requirement (REQ) and
then test (TST) that spec.

The artifacts **will** have simple orthoganal variables that accomplish
the purposes defined in [[REQ-1]].

All artifacts **will** have the following attributes that the user can define:
- name: the name specified by `[ART-name]`
- partof: a string containing artifact names which the artifact is a part of. See [[REQ-partof]]
- text: description of the artifact

This is kept *intentionally* minimal so as to reduce the API space for users
to learn as well as external tools to process.

## Artifact Name
The artifact name should be in a human readable format which allows for simple
categorization of different features.

In order to accomplish this the name will be of the form:
`ART-foo-bar-...` where `ART` is one of the types specified above
and `foo-bar` is an arbitrary category.

Each `"-"` in an artifact name shall be special. They are the primary
separation variable for names and will denote categories and aid in easily
linking artifacts to other artifacts, as defined in [[REQ-partof]]

# REQ-cmd
partof: REQ-1
###
The artifact cmdline interface will be simple and intuitive for those
who have used unix and git. It will contain a small subset of subcommands
that makes usability clean and easy for anyone's use case.

It should have:
- a builtin tutorial
- initialization like `git init`
- listing/filter/etc of artifacts in various forms and with color
- formatting artifact files
- exporting artifact data as html and json
- interface to running [[REQ-web]]

See [[SPC-cmd]].

# REQ-completion
Artifact will track how *complete* and *tested* an item is based on:
- The artifact's type:
    - TST is always as tested as it is completed.
- The completion status of it's children:
    - TST children affect tested% but do not impact completed%
    - SPC children affect completed% but do not impact tested%
    - REQ children affect both completed% and tested%
- Whether it is implemented in code (i.e. `#ART-name` is somewhere in code) or
  implemented using the `done` field.
- The completion/tested status of its parts/children.

In particular
- The `done` field and code links will only be used to calculate completeness
  if the artifact has no children.
- TST artifacts shall only add to the tested% (not completed%) of artifacts
  they are a partof.

# REQ-partof
partof: REQ-1
###
linking should be in a format that is as easy to understand format as possible:
- between artifacts
- with the location of implementation

It is recognized that there are two main concerns:
- ease of writing links
- ease of reading links

Ease of writing links is considered the most important, as reading
links can be accomplished automatically through the cmdline/web ui
interface.

For this reason, artifact uses the "partof" nomenclature for linking.
This allows the user to think "what requirement does the design
document I am currently writing fulfill".

In addition, artifact shall infer links from the name of artifacts
whenever possible, such as artifacts with the same prefix or
the same name of different types.

## Partof
There are three ways to specify that an artifact is partof another artifact:
 1. explicitly through the `partof` attribute
 2. through postfix-name (linkable types)
 3. through prefix-name (parents)

## prefix name (parents)
Parents of an artifact will be automatically linked as partof their children.

**Example**:

`REQ-foo-bar` has "parent" `REQ-foo`, which must exist.

## post-fix name (linkable types)
Artifacts will be linked by their postfix if they are able.

Example: If you define
```
[REQ-foo-bar]
[SPC-foo-bar]
[TST-foo-bar]

[SPC-bar]
```

Then `REQ-foo-bar` will automatically be a partof `SPC-foo-bar`,
`SPC-foo-bar` will be a partof `TST-foo-bar`. `SPC-bar` will not
be a partof anything since it doesn't share a prefix.

# REQ-rpc
partof: REQ-1
###
artifact's server shall provide a JSON-RPC API endpoint for interacting with loaded and processed artifacts.

The API server will serve as the backend for both REQ-web and REQ-tracker

For the Web UI the RPC shall have the following endpionts:
- ReadProject
- UpdateArtifacts
- CreateArtifacts
- DeleteArtifacts
- CreateFiles
- CreateFolders

These are defined in [[SPC-rpc]]

# REQ-security
partof: REQ-1
###
With commands such as `art fmt` and the server being able
to edit files on someone's local machine, the artifact application
shall impose checks at all vulnerable places to ensure
that edits will not be made to files that are outside
the cwd-repo that the user is using.

In addition, post-1.0 releases shall require authentication to edit
files through a web-server (1.0 release will be localhost only)

# REQ-tracker
artifact shall have test tracking functionality that will enable projects
and orginizations to track continuously running integration tests.

Unlike the rest of artifact, this tool **shall not** store it's results in text,
as that would be a tracking, performance and extendability nightmare.
Instead it shall use a database backend and be run through the `serve`
subcommand along with the web-ui.

The data that needs to be stored by the test tracking tool is:
 - test name (i.e. MyTest2)
 - design artifacts it tests (i.e. [TST-foo-bar, TST-baz-boo])
 - timestamp of test
 - version that was tested
    (i.e. 3.2.3.4 commit=154642d49b393e49d9de987685335e9c5a8b2aa7)
 - url/path to view test results and data externally
 - extra binary data

All of this should be kept in a database (probably PostgreSQL)
and be easily searchable.

All of the data shall be accessible through artifact's json-rpc server
endpoint (the same one that hosts and supports the web-ui).

In addition, a section of the web-ui shall be dedicated to viewing
test results, and test artifacts shall include links to their test
data (if it exists). Test results should be easy to filter, graph,
compare and generate reports for in the UI.

# REQ-web
partof: REQ-1
###
artifact **shall** provide a web-based frontend for both reading and editing rendered views of artifacts.

The web-ui will have web-specific features such as:
- dynamic search
- automatic checking of variables (i.e. valid name, partof, etc)
- automatic linking of names
- rendering of `text` markdown

Even with these features however, it will work almost identically to how
the cmdline interface works: editing artifacts will involve editing files
locally.

The overall design is to make it easy for:
- large teams to *view* artifacts
- small teams to *edit* artifacts

Once artifacts have been edited, the team should use revision control
and code review best practices which (hopefully) already exist in
their organization in order to merge the changes into the central
repository.

Requirements of web page:
 - backend and webpage shall be packaged in artifact itself (no external dependencies)
 - webpage should be fast and performant
 - webpage shall provide first-order validation, similar to artifact's first checks
 - editing shall utilize the same workflow as is used when editing text
 - there shall be an option to disable editing (read-only)
 - webpage shall be able to view test execution data
