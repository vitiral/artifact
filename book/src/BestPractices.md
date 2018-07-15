These are a few of the (alpha) best practices when using artifact.

### use-features: use the features of artifact
Artifact contains several useful features. Use them!
- `art check`: makes sure your references are all valid both in code and in your
  artifacts.
- `art fmt`: format your artifacts

### too-many-pieces: Do not break into too many pieces
Artifact is a great tool for breaking up your design, so much so that it is
tempting to specify every detail as it's own artifact

Avoid this:
- SPC-ui-cmd
- SPC-ui-cmd-ls
- SPC-ui-cmd-fmt
- SPC-ui-web
- SPC-ui-web-list
- SPC-ui-web-edit
- SPC-ui-gui
- ... etc...

There is no reason to have the `ui` prefix -- each of these are almost
completely different components and doesn't aid in understanding your
design documents.

Instead, consider having:

- REQ-ui: high level ui requirements
- REQ-cmd: partof=REQ-ui
- REQ-web: partof=REQ-ui
- REQ-gui: partof=REQ-ui

This keeps the breakdown obvious but also keeps the names short.

### short-names: Keep artifact names as short as possible
Nobody wants to read `REQ-ui-web-design-frontend-edit-text`. That level of
nesting is simply not necessary. Something like `REQ-web-edit` will suffice.

Try and combine such detail into a single artifact. Auto-linking is cool, but
don't get carried away! It's okay to specify `partof` when it makes
your tree simpler.

### no-numbers: Use only human readable names
Artifact names should avoid using numbers. If you are tempted to call something
`SPC-foo-1` just break down the different items of `foo` in a bullet point list
in its `text` field and use subnames.

### abbreviations: abbreviate names
Artifact is intended to be used as a cmd line tool, so keeping names short is
very nice.

This is mostly useful for larger projects.

### prefix-acronyms: create acronyms for your prefixes
Use an acronym or abbreviation for your prefixes.

One of the main use cases of short names is for the categories of your
artifacts. For instance, say your storage product had the following features:
- transport
- data availability
- data recovery
- remote replication

It would be annoying to have artifacts like `REQ-transport-drive` or
`REQ-remote_replication-protocol`. Instead, use an acronyms:

- **TP**: transport
- **DA**: data availability
- **DR**: data recovery
- **RR**: remote replication

Now your artifacts look like `REQ-TP-drive` and `REQ-RR-protocol`,
which is much shorter and more readable when looking at a large list.

### uniformity: keep names uniform
Artifact automatically makes `SPC-foo` a partof `REQ-foo` and that is because
they should be related. Make sure your names have meaning so this doesn't
accidentally become a gotcha for your project.

