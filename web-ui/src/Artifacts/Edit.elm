module Artifacts.Edit exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, style, value, href, readonly, rows, cols, id)
import Html.Events exposing (onClick, onInput)
import Regex
import Markdown exposing (toHtml)
import Models exposing (Model, memberArtifact)
import Styles exposing (warning)
import Artifacts.Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)
import Artifacts.View as View
import Artifacts.Select as Select
import Utils exposing (isJust)


{-| regex to search for and replace [[ART-name]]
-}
artifactLinkRegex : Regex.Regex
artifactLinkRegex =
    Regex.caseInsensitive <| Regex.regex <| "\\[\\[(" ++ artifactValidRaw ++ ")\\]\\]"


{-| the entire view

ids: unedited_head

-}
view : Model -> Artifact -> Html AppMsg
view model artifact =
    let
        edit =
            case artifact.edited of
                Just e ->
                    ((if e.revision == artifact.revision then
                        []
                      else
                        [ h1
                            [ class "h1 red"
                            , id "warn_edit_change"
                            ]
                            [ text <|
                                "!! This artifact has been changed"
                                    ++ " by another user since editing"
                                    ++ " started !!"
                            ]
                        ]
                     )
                        ++ [ form model artifact artifact.edited

                           -- Header for original view
                           , h1 [ id "unedited_head" ] [ text "Previous:" ]
                           ]
                    )

                Nothing ->
                    []
    in
        div [ id "edit_view" ]
            ([ nav model artifact ]
                ++ edit
                ++ [ form model artifact Nothing ]
            )


nav : Model -> Artifact -> Html AppMsg
nav model artifact =
    let
        edit =
            if model.settings.readonly then
                []
            else if artifact.edited == Nothing then
                [ editBtn artifact False ]
            else
                [ editBtn artifact True
                , saveBtn artifact
                ]
    in
        div
            [ class "clearfix mb2 white bg-black p1" ]
            ([ listBtn ]
                ++ edit
            )


form : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
form model artifact edited =
    div [ class "m3" ]
        ((nameElements model artifact edited)
            ++ [ div [ class "clearfix py1" ]
                    [ formColumnOne model artifact edited
                    , formColumnTwo model artifact edited
                    ]
               ]
        )


{-| attributes column (non-text)
-}
formColumnOne : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
formColumnOne model artifact edited =
    let
        partofEls =
            [ h3 [] [ text "Partof" ]
            , Select.partof model artifact edited
            ]

        -- don't display parts when editing
        els =
            [ View.completion artifact
            , Select.defined model artifact edited
            , View.implemented model artifact
            ]
                ++ [ if isJust edited then
                        div [] partofEls
                     else
                        div [ class "clearfix py1" ]
                            [ div [ class "col col-6" ] partofEls
                            , div [ class "col col-6" ]
                                [ h3 [] [ text "Parts" ]
                                , View.parts model artifact
                                ]
                            ]
                   ]
    in
        div [ class "col col-6" ] els


{-| Text column
-}
formColumnTwo : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
formColumnTwo model artifact edited =
    div [ class "col col-6" ]
        [ h3 [] [ text "Text" ]
        , selectRenderedBtns model (isJust edited)
        , displayText model artifact edited
        ]



-- NAME


nameElements : Model -> Artifact -> Maybe EditableArtifact -> List (Html AppMsg)
nameElements model artifact edited =
    let
        name_id =
            View.getId "name" artifact edited
    in
        case edited of
            Just e ->
                let
                    warn_els =
                        case checkName model e.name artifact.name of
                            Ok _ ->
                                []

                            Err e ->
                                [ warning e ]

                    editMsg t =
                        ArtifactsMsg <| EditArtifact artifact.id { e | name = t }

                    input_el =
                        input
                            [ class "h1"
                            , name_id
                            , onInput editMsg
                            , value e.name
                            ]
                            []
                in
                    [ input_el ] ++ warn_els

            Nothing ->
                [ h1 [ name_id ] [ text artifact.name.raw ] ]



-- TEXT


{-| select which text view to see (raw or rendered)
ids = {ed_, rd_}*text*{raw, rendered}
-}
selectRenderedBtns : Model -> Bool -> Html AppMsg
selectRenderedBtns model editable =
    let
        newView render =
            let
                view =
                    model.state.textView
            in
                if editable then
                    { view | rendered_edit = render }
                else
                    { view | rendered_read = render }

        getId id_ =
            if editable then
                id ("ed_" ++ id_)
            else
                id ("rd_" ++ id_)

        textView =
            model.state.textView

        ( rendered_clr, raw_clr ) =
            if getRendered model editable then
                ( "black", "gray" )
            else
                ( "gray", "black" )
    in
        span []
            [ button
                -- rendered
                [ class ("btn bold " ++ rendered_clr)
                , getId "select_rendered_text"
                , onClick <| ArtifactsMsg <| ChangeTextViewState <| newView True
                ]
                [ text "rendered" ]
            , button
                -- raw
                [ class ("btn bold " ++ raw_clr)
                , getId "select_raw_text"
                , onClick <| ArtifactsMsg <| ChangeTextViewState <| newView False
                ]
                [ text "raw" ]
            ]


getRendered : Model -> Bool -> Bool
getRendered model edit =
    let
        view =
            model.state.textView
    in
        if edit then
            view.rendered_edit
        else
            view.rendered_read



-- TEXT


displayText : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
displayText model artifact edited =
    if getRendered model (isJust edited) then
        displayRenderedText model artifact edited
    else
        displayRawText model artifact edited


displayRenderedText : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
displayRenderedText model artifact edited =
    let
        id =
            View.getId "rendered_text" artifact edited

        rawText =
            case edited of
                -- show the edited version
                Just e ->
                    e.text

                -- show the original version
                Nothing ->
                    artifact.text

        rendered =
            replaceArtifactLinks model rawText
    in
        toHtml [ id ] rendered


{-| display raw text in a way that can be edited

ids: {rd, ed}*text*(artifact.name.value)

-}
displayRawText : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
displayRawText model artifact edited =
    let
        editedAttrs =
            case edited of
                Just e ->
                    [ (onInput
                        (\t ->
                            (ArtifactsMsg
                                (EditArtifact artifact.id
                                    { e | text = t }
                                )
                            )
                        )
                      )
                    ]

                Nothing ->
                    []

        attrs =
            [ class "h3"

            -- class=h3 otherwise it is really tiny for some reason
            , rows 35
            , cols 80
            , readonly <| not <| isJust edited
            , View.getId "raw_text" artifact edited
            ]
                ++ editedAttrs

        rawText =
            case edited of
                -- show the edited version
                Just e ->
                    e.text

                -- show the original version
                Nothing ->
                    artifact.text
    in
        textarea attrs [ text rawText ]



-- BUTTONS


{-| navigate back to the list page

ids: list

-}
listBtn : Html AppMsg
listBtn =
    button
        [ class "btn regular"
        , id "list"
        , onClick (ArtifactsMsg ShowArtifacts)
        ]
        [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]


{-| start/stop editing

ids: edit/cancel_edit

-}
editBtn : Artifact -> Bool -> Html AppMsg
editBtn artifact in_progress =
    button
        ([ class "btn regular"
         ]
            ++ if in_progress then
                [ id "cancel_edit"
                , onClick (ArtifactsMsg (CancelEditArtifact artifact.id))
                ]
               else
                [ id "edit"
                , onClick (ArtifactsMsg (EditArtifact artifact.id (getEditable artifact)))
                ]
        )
        [ i [ class "fa fa-pencil mr1" ] []
        , text
            (if in_progress then
                "Cancel"
             else
                "Edit"
            )
        ]


{-| save the current edit state. This button does not always exist.

ids: save

-}
saveBtn : Artifact -> Html AppMsg
saveBtn artifact =
    button
        [ class "btn regular"
        , id "save"
        , onClick <| ArtifactsMsg <| SaveArtifact artifact.id
        ]
        [ i [ class "fa fa-floppy-o mr1" ] []
        , text "Save"
        ]



-- HELPERS


{-| get the full url to a single artifact
-}
fullArtifactUrl : Model -> String -> String
fullArtifactUrl model indexName =
    let
        addrName =
            String.toLower (indexNameUnchecked indexName)

        -- super hacky way to get the origin: might fail for files
        -- I tried location.origin... doesn't work for some reason.
        -- neither does location.host + location.pathname
        origin =
            case List.head (String.split "#" model.location.href) of
                Just o ->
                    removeSlashEnd o

                Nothing ->
                    "ERROR-origin-no-head"
    in
        origin ++ "/" ++ artifactsUrl ++ "/" ++ addrName


removeSlashEnd : String -> String
removeSlashEnd path =
    if String.endsWith "/" path then
        removeSlashEnd (String.dropRight 1 path)
    else
        path


{-| replace [[ART-name]] with [ART-name](link)
-}
replaceArtifactLinks : Model -> String -> String
replaceArtifactLinks model text =
    let
        replace : Regex.Match -> String
        replace match =
            case List.head match.submatches of
                Just m ->
                    case m of
                        Just m ->
                            "[" ++ m ++ "](" ++ (fullArtifactUrl model m) ++ ")"

                        Nothing ->
                            "INTERNAL_ERROR"

                Nothing ->
                    "INTERNAL_ERROR"
    in
        Regex.replace Regex.All artifactLinkRegex replace text


{-| Just check that the name is valid and that it doesn't
already exist.
-}
checkName : Model -> String -> Name -> Result String Name
checkName model name original =
    case initName name of
        Ok name ->
            if name == original then
                -- name already exists... because its the same name!
                Ok name
            else if memberArtifact name.value model then
                Err "name already exists"
            else
                Ok name

        Err _ ->
            Err "invalid name"
