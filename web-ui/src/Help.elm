module Help exposing (..)

-- Help pages

import Html exposing (..)
import Html.Attributes exposing (..)
import Markdown exposing (toHtml)
import Messages exposing (HelpPage(..), AppMsg(ShowHelp))
import Artifacts.Nav as Nav
import Utils exposing (strReplace)


-- ICON


helpDesc : HelpPage -> String
helpDesc page =
    case page of
        HelpMain ->
            mainDesc

        HelpName ->
            nameDesc

        HelpParts ->
            partsDesc

        HelpPartof ->
            partofDesc

        HelpText ->
            textDesc

        HelpDefined ->
            definedDesc

        HelpImplemented ->
            implementedDesc

        HelpDone ->
            doneDesc

        HelpEdit ->
            editDesc


getPage : String -> Maybe HelpPage
getPage route =
    case String.toLower route of
        "" ->
            Just HelpMain

        "name" ->
            Just HelpName

        "parts" ->
            Just HelpParts

        "partof" ->
            Just HelpPartof

        "text" ->
            Just HelpText

        "defined" ->
            Just HelpDefined

        "implemented" ->
            Just HelpImplemented

        "done" ->
            Just HelpDone

        "edit" ->
            Just HelpEdit

        _ ->
            Nothing


viewPage : HelpPage -> Html AppMsg
viewPage page =
    let
        rendered =
            toHtml [] (helpDesc page)

        line page =
            li [] [ Nav.helpBtn page True ]

        bottom =
            if page == HelpMain then
                ul []
                    [ line HelpName
                    , line HelpParts
                    , line HelpPartof
                    , line HelpText
                    , line HelpDefined
                    , line HelpImplemented
                    , line HelpDone
                    , line HelpEdit
                    ]
            else
                span [] []
    in
        div [ class "p3" ]
            [ rendered
            , bottom
            ]


mainDesc : String
mainDesc =
    """
# Help Main Page
Artifact is a design doc tool made for developers. It allows anyone to easily
write and link their design docs both to each other and to source code, making
it easy to track how complete their project is.

For a broad overview of artifact and links to the primary user guides, please
go to its [github repo][1].

This is the help page that is shipped with the Web UI of artifact. The Web UI
can exist in multiple different forms:
- rendered as static html (and therefore readonly)
- hosted on a server as readonly
- hosted on a server as editable

When artifact is editable there will be additional functionality for creating,
deleting and editing artifacts. See the [editing help](/#help/edit) for more
information.

Below are pages to help with different aspects of artifact. These pages are linked
through the <i style="class fa fa-info-circle mr1"></i> icon throughout the
artifact Web UI.

[1]: https://github.com/vitiral/artifact
"""


nameDesc : String
nameDesc =
    """
# Help for Artifact Name

The artifact name specifies the type and id of an artifact so
that the artifact can be linked in via its [partof](/#help/partof)
attribute.

The prefix of an artifact must start with one of the following
types, which also defines the limitations on how it can be
linked:
- **REQ**: the artifact is a requirement and can only be partof
  specifications and other requirements.
- **SPC**: the artifact is a specification and can only be
  partof tests and other specifications.
- **TST**: the artifact is a test-design and can only be
  partof other tests.
"""


partofDesc : String
partofDesc =
    """
# Help for Partof
The "partof" field is a user-specifiable attribute for an artifact which is a
list of [names](/#help/name) that are the "parents" of the artifact. When the
attribute is set it makes the artifact a part/child.

**partof** is the opposite of **[parts](/#help/parts)**. Partof is how a user
specifies relation, parts is calculated automatically and is how the "completed"
and "tested" percentages are calculated.

Artifacts can be automatically made a partof another artifact if:
- Their prefix is identical (i.e. `REQ-foo-bar` is automatically a "partof" `REQ-foo`),
  where the `'-'` character is used to determine "prefix".
- It is a subtype of an artifact with the same name (i.e. `SPC-foo` is automatically a
  "partof" `REQ-foo`)

For example:
```
[REQ-foo]
text = '''
This is the parent artifact
'''

[REQ-foo-a]
text = '''
This will be automatically linked to [[REQ-foo]] because
the prefix is the same.
'''

[REQ-bar]
partof = "REQ-foo"
text = '''
This is explicitly a child of [[REQ-foo]] because it
has set its `partof` attribute.
'''

[SPC-foo]
text = '''
This will be automatically linked to [[REQ-foo]] because
SPC is a sub-type of REQ.
'''
```
"""


partsDesc : String
partsDesc =
    """
# Help for Parts (and completed/tested)
The "parts" field is a computed attribute which is a list of artifact
[names](/#help/name) that are the "children" of an artifact.

It is the opposite of [partof](/#help/partof). Parts is calculated
automatically and is how the "completed" and "tested" percentages are
calculated, partof is how a user specifies relation.

Artifacts of type REQ (requirement) can only be "completed" by their REQ and
SPC parts being completed.

Similarily, artifacts of type SPC can be implemented directly in code
and by their SPC parts, but can only be tested by their TST parts being
completed.

> The [implemented](/#help/implemented) and [done](/#help/done) attributes
> are the only ways to mark lower level artifacts as complete.
"""


textDesc : String
textDesc =
    """
# Help For Text
The "text" field is used for describing the purpose of the artifact. The user
writes in the [markdown][1] format, which the web-ui renders.

There is one additional syntax element added, you can use double-brackets
around a valid artifact name to specify a soft link to other artifacts, for
example `[[REQ-example]]`. These links are NOT used to compute %
completion/test, but are automatically checked for validity. If they don't
exist they will be ~~strikethrough~~ and red, and will show up in the
**Checks** tab.

[1]: https://gitbookio.gitbooks.io/markdown/content/
"""


definedDesc : String
definedDesc =
    """
# Help For The "defined" Field
The `defined` field is the path to the toml file where the artifact is actually
defined in text. The format of defining an artifact looks like:

```
[REQ-example]
partof = "REQ-other"
text = '''
Description of artifact.
'''
```

See the [Quick Start][1] for a quick example.

> A url can be automatically generated if the user provides the `--path-url`
> command to either `art serve` or `art export`. See the help documents
> for those commands

[1]: https://github.com/vitiral/artifact/blob/master/docs/QuickStart.md
"""


implementedDesc : String
implementedDesc =
    strReplace "@" "#" """
# Help For The "implementation" Field
The `implementation` field is a computed `file_path[col]` of where the artifact
is implemented in source code.

Artifact automatically searches source code files specified in
`.art/settings.toml` for tokens looking like `@SPC-name`. Only artifacts of
type SPC or TST can be implemented in code.

See the [Quick Start][1] for a quick example.

> A url can be automatically generated if the user provides the `--path-url`
> command to either `art serve` or `art export`. See the help documents
> for those commands

> The [done](/#help/done) field is related to this field.

[1]: https://github.com/vitiral/artifact/blob/master/docs/QuickStart.md
"""


doneDesc : String
doneDesc =
    """
# Help For "done" Field
The "done" field is a shortcut field for marking any artifact as
100% tested and completed. If the artifact has any parts, it will
still use those to calculate its completion/tested percentages as well.

> In general, it is recommended that you use [implemented](/#help/implemented)
> whenever possible instead.

The "done" field is a simple string of raw text. It can be a reason, a url to
another project or anything else that the user wants to give as justification
for why they are marking the arifact as done.

This is typically used for things such as:
- Definitions: when you are defining things like the programming language you
  will use, or the definition of terms, etc
- External projects: sometimes an external project will fulfill a requirement,
  specification or test. It's okay to give a link to that project.
- TODOs: especially if you are writing design documents for a project that
  already exists it can be helpful to set `done="TODO"` to eventually link
  it to its implementation.
"""


editDesc : String
editDesc =
    """
# Help for Editing Artifacts
There are several important points to understand about editing artifacts:
- Artifact is fundamentally a text-based tool (which allows it to be revision
  controllable using tools like git), so edits are always saved in the text
  files where the server was run. This means that when you save an artifact it
  will change information in a text file.
- Editing artifacts works concurrently but the operation is fairly slow
  (compared to typical database operations). The intended workflow is for only a
  small team to be editing artifacts off a single server, and then use revision
  control to code review and merge that team's efforts into the full design
  repository.
- There is (currently) no undo operation for edits. However, since the
  design documents are saved in a text file edits can be undone using revision
  control.

## Editing UI
Existing artifacts have an **Edit** button which allows the user to change
anything about the artifact. To commit changes (and save them in text), press
the **Save** button. To cancel changes the press the **Cancel** button.

An artifact can be deleted using the **Delete** button on its edit page.

You can also create a new artifact using the **Create** button.
"""
