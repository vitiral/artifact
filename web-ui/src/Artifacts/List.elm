module Artifacts.List exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, width)
import Html.Events exposing (onClick)

import Messages exposing (AppMsg(..))
import Models exposing (Model)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (Artifact, ArtifactConfig)
import Artifacts.View as View


view : Model -> List Artifact -> Html AppMsg
view model artifacts =
  div []
    [ nav artifacts
    , list model artifacts
    ]

nav : List Artifact -> Html AppMsg
nav artifacts = 
  div [ class "clearfix mb2 white bg-black" ]
    [ div [ class "left p2" ] [ text "Artifacts" ]
    ]

base_width = 400
w1 = width base_width
w2 = width (base_width * 2)
w3 = width (base_width * 3)

list : Model -> List Artifact -> Html AppMsg
list model artifacts =
  let
    len = List.length artifacts
    model_list = List.repeat len model
    key = \a b -> compare a.name b.name
    sorted = List.sortWith key artifacts
  in
    div [ class "p2" ]
      [ table []
        [ thead [] -- thead == table-header object
          [ tr []  -- table row
            [ th [ class "bold", w1] [ text "Compl" ]
            , th [ class "bold", w1 ] [ text "Tested" ]
            , th [ class "bold", w2 ] [ text "Name" ]
            , th [ class "bold", w2 ] [ text "Parts" ]
            , th [ class "bold", w2 ] [ text "Part Of" ]
            , th [ class "bold", w2 ] [ text "Def At" ]
            , th [ class "bold", w2 ] [ text "Impl At" ]
            , th [ class "bold", w3 ] [ text "Text" ]
            ]
          ]
        , tbody [] (List.map2 artifactRow model_list sorted)
        ]
      ]

artifactRow : Model -> Artifact -> Html AppMsg
artifactRow model artifact =
  let
    s = Html.Attributes.style [("vertical-align", "top")]
    partsConfig = (\config value -> { config | partsExpanded = value})
    partsHtml = expandable 
      artifact.config.partsExpanded 
      model artifact View.parts partsConfig

    partofConfig = (\config value -> { config | partofExpanded = value})
    partofHtml = expandable 
      artifact.config.partofExpanded 
      model artifact View.partof partofConfig

    pathConfig = (\config value -> { config | pathExpanded = value})
    pathView = (\model artifact -> text artifact.path)
    pathHtml = expandable 
      artifact.config.pathExpanded 
      model artifact pathView pathConfig

    locConfig = (\config value -> { config | locExpanded = value})
    locHtml = expandable 
      artifact.config.locExpanded 
      model artifact View.implementedBasic locConfig

    textConfig = (\config value -> { config | textExpanded = value})
    textView = (\model artifact -> div [] [ View.textPiece model artifact ])
    textHtml = expandable 
      artifact.config.textExpanded 
      model artifact textView textConfig
  in
    tr []
      [ td [s, w1] [ View.completedPerc artifact ]
      , td [s, w1] [ View.testedPerc artifact ]
      , td [s, w2] [ View.seeArtifact model artifact ]
      , td [s, w2] [ partsHtml ]
      , td [s, w2] [ partofHtml ]
      , td [s, w2] [ pathHtml ]
      , td [s, w2] [ locHtml ]
      , td [s, w3] [ textHtml ]
      ]

-- given the app model + an artifact, as well as
-- the view when expanded and the function to perform
-- on the config (config + expanded = config_expanded)
-- return a view that enables all these things
expandable : 
 Bool -> Model -> Artifact 
  -> (Model -> Artifact -> Html AppMsg)
  -> (ArtifactConfig -> Bool -> ArtifactConfig)
  -> Html AppMsg
expandable expanded model artifact html setConfig = 
  if expanded then
    div []
      [ button 
        [ class "btn regular"
        , onClick (ArtifactsMsg <| SetExpand artifact.id setConfig False)
        ] [ i [ class "fa fa-minus-square-o m0" ] [] ]
      , html model artifact
      ]
  else
    div [] 
      [ button 
        [ class "btn regular"
        , onClick (ArtifactsMsg <| SetExpand artifact.id setConfig True)
        ] [ i [ class "fa fa-plus-square-o m0" ] [] ]
      ]
