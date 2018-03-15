module Artifacts.Edit exposing (..)

import Maybe exposing (withDefault)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput)
import Models exposing (Model, getArtifact, memberArtifact, getCreateArtifact)
import Styles exposing (warning)
import Artifacts.Models exposing (..)
import Messages exposing (createUrl, AppMsg(..), HelpPage(..))
import Artifacts.Messages exposing (..)
import Artifacts.View as View
import Artifacts.Nav as Nav
import Artifacts.Select as Select


view : Model -> EditOption -> Html AppMsg
view model choice =
    let
        option =
            EditChoice choice

        nav =
            Nav.bar model <| Nav.editBar model option

        headName =
            span [ id "editing_head" ]
                [ Nav.helpBtn HelpEdit False
                , text "Editing"
                ]

        head =
            div [ class "h1" ] <|
                [ headName ]
                    ++ (editName model choice)

        done =
            div [] <| doneFieldEdit model choice

        textEl =
            displayText model choice

        define =
            div [ class "clearfix py1" ]
                [ div [ class "col col-4" ] [ Select.defined model option, done ]
                , div [ class "col col-4" ] [ Select.partof model option ]
                ]

        elems =
            [ [ nav ]
            , View.revisionWarnings model option
            , [ head ]
            , [ define ]
            , textEl
            ]
    in
        div [ View.viewIdAttr option ] (List.concat elems)


{-| The text is displayed as the editable portion on the left
and it's rendered counterpart on the right.
-}
displayText : Model -> EditOption -> List (Html AppMsg)
displayText model choice =
    let
        edit =
            displayEditableText model choice

        rendered =
            View.displayRenderedText model (EditChoice choice)

        out =
            div [ class "clearfix border" ]
                [ div [ class "col border" ] edit
                , div [ class "col border" ] [ rendered ]
                ]
    in
        [ out ]


editName : Model -> EditOption -> List (Html AppMsg)
editName model choice =
    let
        edited =
            getEdited choice

        warn_els =
            case Nav.checkName model edited.name choice of
                Ok _ ->
                    []

                Err e ->
                    [ warning e ]

        editMsg t =
            ArtifactsMsg <|
                EditArtifact <|
                    setEdited choice { edited | name = t }

        input_el =
            [ Nav.helpBtn HelpName False
            , input
                [ class "h1"
                , View.idAttr "name" (EditChoice choice)
                , onInput editMsg
                , value edited.name
                ]
                []
            ]
    in
        List.append input_el warn_els


{-| display raw text in a way that can be edited
-}
displayEditableText : Model -> EditOption -> List (Html AppMsg)
displayEditableText model choice =
    let
        edited =
            getEdited choice

        changedMsg t =
            ArtifactsMsg <|
                EditArtifact <|
                    setEdited choice { edited | text = t }

        attrs =
            [ rows 35
            , cols 80
            , readonly False
            , View.idAttr "raw_text" (EditChoice choice)
            , onInput changedMsg
            ]
    in
        [ textarea attrs [ text edited.text ] ]


{-| Edit the done field
-}
doneFieldEdit : Model -> EditOption -> List (Html AppMsg)
doneFieldEdit model choice =
    let
        edited =
            getEdited choice

        editMsg t =
            let d = case t of
                "" -> Nothing
                _ -> Just t
            in
                setEdited choice { edited | done = d }
                    |> EditArtifact
                    |> ArtifactsMsg
    in
        [ span [ class "bold" ]
            [ Nav.helpBtn HelpDone False
            , text "Define as done:"
            ]
        , input
            [ View.idAttr "done" <| EditChoice choice
            , onInput editMsg
            , value (withDefault "" edited.done)
            ]
            []
        ]
