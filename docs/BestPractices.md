These are a few of the (alpha) best practices when using artifact.

### use-features: use the features of artifact

Artifact contains several useful features. Use them!

- `art check`: makes sure your references are all valid both in code and in your
  artifacts.
- `art fmt`: format your artifacts

### too-many-pieces: Do not break into too many pieces
artifact is a great tool for breaking up your design, so much so that it is
tempting to specify every detail as it's own requirement.

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
requirements.

Instead, consider having:

- REQ-ui: high level ui requirements
- REQ-cmd: partof=REQ-ui
- REQ-web: partof=REQ-ui
- REQ-gui: partof=REQ-ui

This keeps the breakdown obvious but also keeps the names short.

### short-names: Keep artifact names as short as possible
Nobody wants to read `REQ-ui-web-design-frontend-edit-text`. That level of
nesting is simply not necessary. Something like `REQ-web-edit` will suffice.

Try and combine such detail into a single artifact. Linking is cool, but don't
get carried away!

### useful-numbers: Use simple numbers as the first value
### for special requirements
In general, numbers as artifact names should be avoided. However, for categorizing
they are very useful as they keep the name short and simple.

The following nomenclature is recommended: `ART-X-topic` where
`X` is one of:
**0. definition and process**: This comes before purpose or high
    level specification and is where you define the rules for your
    requirements. For example, `REQ-0-assertions` might specify
    what **shall, will and should** will mean.
**1. purpose requirements**: These detail at the highest possible level
    what the project is for and are always REQuirements
N. Use additional ones as needed, but be wary! Not many projects need more.

### no-numbers: Use only human readable names
Except for as noted in the previous section, artifact names should avoid using
numbers. If you are tempted to call something `SPC-foo-1` just break down the
different items of `foo` in a bullet point list in it's `text` field.

Then, if you want to reference it in your source code,
just say something like `see SPC-foo.1`.

> Note: Periods (`.`) are actually invalid characters in artifact, and there
> is a possibly that the above will actually be seen to mean something
> special when linked in source code.

### abbreviations: abbreviate names
artifact is intended to be used as a cmd line tool, so keeping names short is
very nice.

This is mostly useful for larger projects.

It is also good to define all your abbreviations in `REQ-0-abbreviations`

### prefix-acronyms: create acronyms for your prefixes
Use an acronym or abbreviation for your prefixes.

One of the main use cases of short names is for the categories
of your artifacts. For instance, say your storage product had the
following features:
- transport
- data availability
- data recovery
- remote replication

It would be annoying to have artifacts like `REQ-transport-drive`
or `REQ-remote_replication-protocol`. Instead, use an acronyms:

- **TP**: transport
- **DA**: data availability
- **DR**: data recovery
- **RR**: remote replication

Now your requirements look like `REQ-TP-drive` and `REQ-RR-protocol`,
which is much shorter and more readable when looking at a large list.

### uniformity: keep names uniform
artifact automatically makes `SPC-foo` a partof `REQ-foo` and that is because
they should be related. Make sure your requirement names have meaning.

