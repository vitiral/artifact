module Artifacts.Edit exposing (..)

import Dict
import Html exposing (..)
import Html.Attributes exposing (class, style, value, href, readonly, rows, cols, id)
import Html.Events exposing (onClick, onInput)
import Regex
import Markdown exposing (toHtml)
import Models exposing (Model, getArtifact, memberArtifact)
import Styles exposing (warning)
import Artifacts.Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)
import Artifacts.View as View
import Artifacts.Select as Select
import Artifacts.Nav as Nav


{-| regex to search for and replace [[ART-name]]
-}
artifactLinkRegex : Regex.Regex
artifactLinkRegex =
    Regex.caseInsensitive <| Regex.regex <| "\\[\\[(" ++ artifactValidRaw ++ ")\\]\\]"


{-| the entire view

This is the ONLY place that used the "artifact id" hack,
where id=0 corresponds to "create"
-}
view : Model -> ArtifactId -> Html AppMsg
view model art_id =
    let
        -- FIXME
        artifact = case Dict.get art_id model.artifacts of
            Just a -> a
            Nothing -> Debug.crash "FIXME"

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
                        ++ [ form model <| EditChoice (ChangeChoice artifact e)

                           -- Header for original view
                           , h1 [ id "unedited_head" ] [ text "Previous:" ]
                           ]
                    )

                Nothing ->
                    []

        nav =
            if model.settings.readonly then
                Nav.bar <| Nav.readBar model artifact
            else
                Nav.bar <| Nav.editBar model artifact
    in
        div [ id "edit_view" ]
            ([ nav ]
                ++ edit
                ++ [ form model <| ReadChoice artifact ]
            )


form : Model -> ViewOption -> Html AppMsg
form model option =
    div [ class "m3" ]
        ((nameElements model option)
            ++ [ div [ class "clearfix py1" ]
                    [ formColumnOne model option
                    , formColumnTwo model option
                    ]
               ]
        )


{-| attributes column (non-text)
-}
formColumnOne : Model -> ViewOption -> Html AppMsg
formColumnOne model option =
    let
        partofEls =
            [ h3 [] [ text "Partof" ]
            , Select.partof model option
            ]

        elements =
            case option of
                ReadChoice artifact ->
                    -- display all information
                    [ View.completion artifact
                    , Select.defined model option
                    , View.implemented model artifact
                    , div [ class "clearfix py1" ]
                        [ div [ class "col col-6" ] partofEls
                        , div [ class "col col-6" ]
                            [ h3 [] [ text "Parts" ]
                            , View.parts model artifact
                            ]
                        ]
                    ]

                EditChoice _ ->
                    -- only display editable information
                    [ Select.defined model option ] ++ partofEls
    in
        div [ class "col col-6" ] elements


{-| Text column
-}
formColumnTwo : Model -> ViewOption -> Html AppMsg
formColumnTwo model option =
    div [ class "col col-6" ]
        [ h3 [] [ text "Text" ]
        , selectRenderedBtns model option
        , displayText model option
        ]



-- NAME


nameElements : Model -> ViewOption -> List (Html AppMsg)
nameElements model option =
    let
        name_id =
            View.idAttr "name" option
    in
        case option of
            ReadChoice artifact ->
                [ h1 [ name_id ] [ text artifact.name.raw ] ]

            EditChoice choice ->
                let
                    edited = getEdited choice

                    warn_els =
                        case Nav.checkName model edited.name choice of
                            Ok _ ->
                                []

                            Err e ->
                                [ warning e ]

                    editMsg t =
                        ArtifactsMsg <| EditArtifact (getArtifactId choice) { edited | name = t }

                    input_el =
                        input
                            [ class "h1"
                            , name_id
                            , onInput editMsg
                            , value edited.name
                            ]
                            []
                in
                    [ input_el ] ++ warn_els



-- TEXT


{-| select which text view to see (raw or rendered)
ids = {ed_, rd_}*text*{raw, rendered}
-}
selectRenderedBtns : Model -> ViewOption -> Html AppMsg
selectRenderedBtns model option =
    let
        newView render =
            let
                view =
                    model.state.textView
            in
                if isRead option then
                    { view | rendered_read = render }
                else
                    { view | rendered_edit = render }

        textView =
            model.state.textView

        ( rendered_clr, raw_clr ) =
            if isTextRendered model option then
                ( "black", "gray" )
            else
                ( "gray", "black" )
    in
        span []
            [ button
                -- rendered
                [ class ("btn bold " ++ rendered_clr)
                , id <| (View.idPrefix option) ++ "select_rendered_text"
                , onClick <| ArtifactsMsg <| ChangeTextViewState <| newView True
                ]
                [ text "rendered" ]
            , button
                -- raw
                [ class ("btn bold " ++ raw_clr)
                , id <| (View.idPrefix option) ++ "select_raw_text"
                , onClick <| ArtifactsMsg <| ChangeTextViewState <| newView False
                ]
                [ text "raw" ]
            ]


isTextRendered : Model -> ViewOption -> Bool
isTextRendered model option =
    let
        view =
            model.state.textView
    in
        if isRead option then
            view.rendered_read
        else
            view.rendered_edit



-- TEXT


displayText : Model -> ViewOption -> Html AppMsg
displayText model option =
    if isTextRendered model option then
        displayRenderedText model option
    else
        displayRawText model option


displayRenderedText : Model -> ViewOption -> Html AppMsg
displayRenderedText model option =
    let
        rawText =
            case option of
                ReadChoice a ->
                    a.text

                EditChoice c ->
                    (getEdited c).text

        rendered =
            replaceArtifactLinks model rawText
    in
        toHtml [ View.idAttr "rendered_text" option ] rendered


{-| display raw text in a way that can be edited
-}
displayRawText : Model -> ViewOption -> Html AppMsg
displayRawText model option =
    let
        ( rawText, editedAttrs ) =
            case option of
                ReadChoice artifact ->
                    ( artifact.text, [] )

                EditChoice choice ->
                    let
                        edited = getEdited choice

                        changedMsg t =
                            ArtifactsMsg <|
                                EditArtifact (getArtifactId choice) { edited | text = t }
                    in
                        ( edited.text, [ onInput changedMsg ] )

        attrs =
            [ class "h3"

            -- class=h3 otherwise it is really tiny for some reason
            , rows 35
            , cols 80
            , readonly <| isRead option
            , View.idAttr "raw_text" option
            ]
    in
        textarea (attrs ++ editedAttrs) [ text rawText ]



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
