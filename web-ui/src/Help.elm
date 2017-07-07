module Help exposing (..)

-- Help pages

import Html exposing (..)
import Html.Attributes exposing (..)
import Markdown exposing (toHtml)
import Messages exposing (HelpPage(..), AppMsg(ShowHelp))
import Artifacts.Nav as Nav


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

It is illegal to have recursive partof links. See the
[check](/#help/check) page for more information.
"""


partsDesc : String
partsDesc =
    """
# Help for Parts

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
"""


editDesc : String
editDesc =
    """
# Help for Editing Artifacts
Existing artifacts will have an **Edit** button which allows the user to change
anything about the artifact. To commit changes the user presses the **Save**
button. To cancel changes the user presses the **Cancel** button.


"""
