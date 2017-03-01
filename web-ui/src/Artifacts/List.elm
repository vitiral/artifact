module Artifacts.List exposing (..)

import Dict
import String
import Regex

import Html exposing (..)
import Html.Attributes exposing (class, type_, width, id, readonly, cols, size)
import Html.Events exposing (onClick, onInput)

import Messages exposing (AppMsg(..))
import Models exposing (Model)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (Artifact, Artifacts, Columns, Search)
import Artifacts.View as View


view : Model -> Artifacts -> Html AppMsg
view model artifacts =
  div []
    [ nav artifacts
    -- select / search bar
    , div [ class "clearfix py1" ]
      [ span [ class "left border" ] [ select model ]
      , span [ class "right border" ] [ searchBar model ]
      ]
    , list model
    ]

-- navigation toolbar
nav : Artifacts -> Html AppMsg
nav artifacts = 
  div [ class "clearfix mb2 white bg-black" ]
    [ div [ class "left p2" ] [ text "Artifacts" ]
    ]

-- SELECT COL

-- select which attrs to view
select : Model -> Html AppMsg
select model =
  let
    cols = model.state.columns
  in
    span []
      [ span [ class "bold" ] [ text "View: " ]
      , selectColBtn "parts" cols.parts (\s -> { cols | parts = s })
      , selectColBtn "partof" cols.partof (\s -> { cols | partof = s })
      , selectColBtn "text" cols.text (\s -> { cols | text = s })
      , selectColBtn "def-at" cols.path (\s -> { cols | path = s })
      , selectColBtn "done" cols.loc (\s -> { cols | loc = s })
      ]

-- Button which takes the current value and a method
-- for how to set the Column
selectColBtn : String -> Bool -> ( Bool -> Columns ) -> Html AppMsg
selectColBtn name visible setter =
  let
    color = if visible then
      "black"
    else
      "gray"
  in
    button
      [ class ("btn bold " ++ color)
      , onClick <| ArtifactsMsg <| ColumnsChanged <|setter <| not visible
      ]
      [ text name ]


-- SEARCH

searchBar : Model -> Html AppMsg
searchBar model =
  let
    sch = model.state.search
    regex = sch.regex
  in
    span []
      [ span [ class "bold" ] [ text "Search: " ]
      , searchAttrBtn "name" sch.name (\s -> { sch | name = s })
      , searchAttrBtn "parts" sch.parts (\s -> { sch | parts = s })
      , searchAttrBtn "partof" sch.partof (\s -> { sch | partof = s })
      , searchAttrBtn "text" sch.text (\s -> { sch | text = s })
      , searchInput sch
      , searchAttrBtn "as-regex" sch.regex (\s -> { sch | regex = s })
      ]

searchInput : Search -> Html AppMsg
searchInput sch =
  input 
    [ size 40
    , readonly False
    , onInput (\t -> (ArtifactsMsg (SearchChanged { sch | pattern = t })))
    ] 
    [ text sch.pattern ]

searchAttrBtn : String -> Bool -> ( Bool -> Search ) -> Html AppMsg
searchAttrBtn name sel setter =
  let
    color = if sel then
      "black"
    else
      "gray"
  in
    button
      [ class ("btn bold " ++ color)
      , onClick <| ArtifactsMsg <| SearchChanged <| setter (not sel)
      ]
      [ text name ]

-- Apply search settings to artifacts
search : Model -> List Artifact
search model =
  let
    sch = model.state.search
    pat_str = if sch.regex then
      sch.pattern
    else
      Regex.escape sch.pattern

    pat = Regex.caseInsensitive (Regex.regex pat_str)

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
    filter a 
      = trySearch sch.name a.name.value
      || trySearchList sch.parts (List.map (\n -> n.value) a.parts)
      || trySearchList sch.partof (List.map (\n -> n.value) a.partof)
      || trySearch sch.text a.text
  in
    List.filter filter (Dict.values model.artifacts)

-- LIST

base_width : Int
base_width = 400

w1 : Attribute msg
w1 = width base_width

w2 : Attribute msg
w2 = width (base_width * 2)

w3 : Attribute msg
w3 = width (base_width * 3)

list : Model -> Html AppMsg
list model =
  let
    key = \a b -> compare a.name.value b.name.value
    artifacts = search model
    sorted = List.sortWith key artifacts
    cls = class "bold left-align"

    model_list = List.repeat (List.length artifacts) model

    columns = model.state.columns
    header = 
      [ th [ cls, w1, id "th_completed" ] [ text "Impl" ]
      , th [ cls, w1, id "th_tested"    ] [ text "Test" ]
      , th [ cls, w2, id "th_name"      ] [ text "Name" ]
      ] 
      ++ (if columns.parts then
        [ th [ cls, w2, id "th_parts"     ] [ text "Parts" ] ]
      else
        [ ])
      ++ (if columns.partof then
        [ th [ cls, w2, id "th_partof"     ] [ text "Partof" ] ]
      else
        [ ])
      ++ (if columns.text then
        [ th [ cls, w2, id "th_text"     ] [ text "Text" ] ]
      else
        [ ])
      ++ (if columns.path then
        [ th [ cls, w2, id "th_path"     ] [ text "Def-At" ] ]
      else
        [ ])
      ++ (if columns.loc then
        [ th [ cls, w2, id "th_done"     ] [ text "Done" ] ]
      else
        [ ])

  in
    div [ class "p2" ]
      [ table [ ]
        [ thead [] -- thead == table-header object
          [ tr []  -- table row
            header
          ]
        , tbody [] (List.map2 artifactRow model_list sorted)
        ]
      ]

artifactRow : Model -> Artifact -> Html AppMsg
artifactRow model artifact =
  let
    s = Html.Attributes.style [("vertical-align", "top")]
    cls = class "border"

    columns = model.state.columns
    cols = 
      [ td [s, cls, w1] [ View.completedPerc artifact ]
      , td [s, cls, w1] [ View.testedPerc artifact ]
      , td [s, cls, w2] [ View.seeArtifact model artifact ]
      ]
      ++ (if columns.parts then
        [ td [s, cls, w2] [ View.parts model artifact ] ]
      else
        [])
      ++ (if columns.partof then
        [ td [s, cls, w2] [ View.partof model artifact ] ]
      else
        [])
      ++ (if columns.text then
        [ td [s, cls, w2] [ View.textPiece model artifact ] ]
      else
        [])
      ++ (if columns.path then
        [ td [s, cls, w2] [ text artifact.path ] ]
      else
        [])
      ++ (if columns.loc then
        [ td [s, cls, w2] [ View.implementedBasic model artifact ] ]
      else
        [])
  in
    tr [ id ("row_" ++ artifact.name.value) ] cols
