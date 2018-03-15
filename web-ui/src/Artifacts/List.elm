module Artifacts.List exposing (..)

import Dict
import Regex
import Html exposing (..)
import Html.Attributes exposing (class, type_, width, id, readonly, cols, size, value)
import Html.Events exposing (onClick, onInput)
import Messages exposing (AppMsg(..))
import Models exposing (Model)
import Artifacts.Messages exposing (..)
import Artifacts.Models
    exposing
        ( Artifact
        , Artifacts
        , Columns
        , Search
        , ViewOption(ReadChoice)
        )
import Artifacts.View as View
import Artifacts.Select as Select
import Artifacts.Nav as Nav


view : Model -> Artifacts -> Html AppMsg
view model artifacts =
    div [ id "list_view" ]
        [ Nav.bar model <| Nav.listBar model

        -- select / search bar
        , div [ class "clearfix py1" ]
            [ span [ class "left border" ] [ select model ]
            , span [ class "right border" ] [ searchBar model ]
            ]
        , list model
        ]


{-| SELECT COL
select which attrs to view

ids: select_col_{arts, partof, text, file, done}

-}
select : Model -> Html AppMsg
select model =
    let
        cols =
            model.state.columns
    in
        span []
            [ span [ class "bold" ] [ text "View: " ]
            , selectColBtn "parts" cols.parts (\s -> { cols | parts = s })
            , selectColBtn "partof" cols.partof (\s -> { cols | partof = s })
            , selectColBtn "text" cols.text (\s -> { cols | text = s })
            , selectColBtn "file" cols.file (\s -> { cols | file = s })
            , selectColBtn "done" cols.loc (\s -> { cols | loc = s })
            ]


{-| Button which takes the current value and a method
for how to set the Column
-}
selectColBtn : String -> Bool -> (Bool -> Columns) -> Html AppMsg
selectColBtn name visible setter =
    let
        color =
            if visible then
                "black"
            else
                "gray"
    in
        button
            [ class ("btn bold " ++ color)
            , id <| "select_col_" ++ name
            , onClick <| ArtifactsMsg <| ChangeColumns <| setter <| not visible
            ]
            [ text name ]


{-| SEARCH
specify what to search for

ids: search_input, search_attr_{name, parts, partof, text}

-}
searchBar : Model -> Html AppMsg
searchBar model =
    let
        sch =
            model.state.search
    in
        span []
            [ i [ class "fa fa-search ml1 mr1" ] []
            , searchAttrBtn "name" sch.name (\s -> { sch | name = s })
            , searchAttrBtn "parts" sch.parts (\s -> { sch | parts = s })
            , searchAttrBtn "partof" sch.partof (\s -> { sch | partof = s })
            , searchAttrBtn "text" sch.text (\s -> { sch | text = s })
            , searchInput sch
            ]


searchInput : Search -> Html AppMsg
searchInput sch =
    input
        [ id "search_input"
        , size 40
        , readonly False
        , onInput (\t -> (ArtifactsMsg (ChangeSearch { sch | pattern = t })))
        , value sch.pattern
        ]
        []


searchAttrBtn : String -> Bool -> (Bool -> Search) -> Html AppMsg
searchAttrBtn name sel setter =
    let
        color =
            if sel then
                "black"
            else
                "gray"
    in
        button
            [ class ("btn bold " ++ color)
            , id <| "search_" ++ name
            , onClick <| ArtifactsMsg <| ChangeSearch <| setter (not sel)
            ]
            [ text name ]


{-| Apply search settings to artifacts
-}
search : Model -> List Artifact
search model =
    let
        sch =
            model.state.search

        pat =
            Regex.caseInsensitive <| Regex.regex sch.pattern

        -- first arg is whether to even try
        trySearch : Bool -> String -> Bool
        trySearch try value =
            if try then
                Regex.contains pat value
            else
                False

        -- search a list... maybe
        trySearchList : Bool -> List String -> Bool
        trySearchList try values =
            if try then
                List.any (\v -> Regex.contains pat v) values
            else
                False

        -- filter out the artifact per the settings
        filter : Artifact -> Bool
        filter a =
            trySearch sch.name a.name.value
                || trySearchList sch.parts (List.map (\n -> n.value) a.parts)
                || trySearchList sch.partof (List.map (\n -> n.value) a.partof)
                || trySearch sch.text a.text
    in
        List.filter filter (Dict.values model.artifacts)



-- LIST


base_width : Int
base_width =
    400


w1 : Attribute msg
w1 =
    width base_width


w2 : Attribute msg
w2 =
    width (base_width * 2)


w3 : Attribute msg
w3 =
    width (base_width * 3)


list : Model -> Html AppMsg
list model =
    let
        key =
            \a b -> compare a.name.value b.name.value

        artifacts =
            search model

        sorted =
            List.sortWith key artifacts

        cls =
            class "bold left-align"

        model_list =
            List.repeat (List.length artifacts) model

        columns =
            model.state.columns

        header =
            [ th [ cls, w1, id "th_completed" ] [ text "Impl" ]
            , th [ cls, w1, id "th_tested" ] [ text "Test" ]
            , th [ cls, w2, id "th_name" ] [ text "Name" ]
            ]
                ++ (if columns.parts then
                        [ th [ cls, w2, id "th_parts" ] [ text "Parts" ] ]
                    else
                        []
                   )
                ++ (if columns.partof then
                        [ th [ cls, w2, id "th_partof" ] [ text "Partof" ] ]
                    else
                        []
                   )
                ++ (if columns.text then
                        [ th [ cls, w2, id "th_text" ] [ text "Text" ] ]
                    else
                        []
                   )
                ++ (if columns.file then
                        [ th [ cls, w2, id "th_file" ] [ text "Def" ] ]
                    else
                        []
                   )
                ++ (if columns.loc then
                        [ th [ cls, w2, id "th_done" ] [ text "Done" ] ]
                    else
                        []
                   )
    in
        div [ class "p2" ]
            [ table []
                [ thead [ id "list_head" ]
                    -- thead == table-header object
                    [ tr [] header
                    ]
                , tbody [ id "list_items" ] (List.map2 artifactRow model_list sorted)
                ]
            ]


artifactRow : Model -> Artifact -> Html AppMsg
artifactRow model artifact =
    let
        s =
            Html.Attributes.style [ ( "vertical-align", "top" ) ]

        cls =
            class "border"

        columns =
            model.state.columns

        cols =
            [ td [ s, cls, w1 ] [ View.completedPerc artifact ]
            , td [ s, cls, w1 ] [ View.testedPerc artifact ]
            , td [ s, cls, w2 ] [ View.seeArtifact model artifact ]
            ]
                ++ (if columns.parts then
                        [ td [ s, cls, w2 ] [ View.parts model artifact ] ]
                    else
                        []
                   )
                ++ (if columns.partof then
                        [ td [ s, cls, w2 ] [ Select.partof model (ReadChoice artifact) ] ]
                    else
                        []
                   )
                ++ (if columns.text then
                        [ td [ s, cls, w2 ] [ View.textPiece model artifact ] ]
                    else
                        []
                   )
                ++ (if columns.file then
                        [ td [ s, cls, w2 ] [ text artifact.file ] ]
                    else
                        []
                   )
                ++ (if columns.loc then
                        [ td [ s, cls, w2 ] [ View.implementedBasic model artifact ] ]
                    else
                        []
                   )
    in
        tr [ id ("row_" ++ artifact.name.value) ] cols
