module Artifacts.List exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import Messages exposing (AppMsg(..))
import Models exposing (Settings)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (Artifact, ArtifactConfig)
import Artifacts.View as View

view : Settings -> List Artifact -> Html AppMsg
view settings artifacts =
  div []
    [ nav artifacts
    , list settings artifacts
    ]

nav : List Artifact -> Html AppMsg
nav artifacts = 
  div [ class "clearfix mb2 white bg-black" ]
    [ div [ class "left p2" ] [ text "Artifacts" ]
    ]

list : Settings -> List Artifact -> Html AppMsg
list settings artifacts =
  let
    len = List.length artifacts
    settings_list = List.repeat len settings
  in
    div [ class "p2" ]  -- consider class=overflow-scroll
      [ table []
        [ thead [] -- thead == table-header object
          [ tr []  -- table row
            [ th [ class "bold" ] [ text "Compl" ]
            , th [ class "bold" ] [ text "Tested" ]
            , th [ class "bold" ] [ text "Name" ]
            , th [ class "bold" ] [ text "Parts" ]
            , th [ class "bold" ] [ text "Part Of" ]
            , th [ class "bold" ] [ text "Def At" ]
            , th [ class "bold" ] [ text "Impl At" ]
            , th [ class "bold" ] [ text "Text" ]
            ]
          ]
        , tbody [] (List.map2 artifactRow settings_list artifacts)
        ]
      ]

artifactRow : Settings -> Artifact -> Html AppMsg
artifactRow settings artifact =
  let
    s = Html.Attributes.style [("vertical-align", "top")]
    partsConfig = (\config value -> { config | partsExpanded = value})
    partsHtml = expandable 
      artifact.config.partsExpanded 
      settings artifact View.parts partsConfig

    partofConfig = (\config value -> { config | partofExpanded = value})
    partofHtml = expandable 
      artifact.config.partofExpanded 
      settings artifact View.partof partofConfig

    pathConfig = (\config value -> { config | pathExpanded = value})
    pathView = (\settings artifact -> text artifact.path)
    pathHtml = expandable 
      artifact.config.pathExpanded 
      settings artifact pathView pathConfig

    locConfig = (\config value -> { config | locExpanded = value})
    locHtml = expandable 
      artifact.config.locExpanded 
      settings artifact View.implementedBasic locConfig

    textConfig = (\config value -> { config | textExpanded = value})
    textView = (\settings artifact -> div [] [ View.textPiece settings artifact ])
    textHtml = expandable 
      artifact.config.textExpanded 
      settings artifact textView textConfig
  in
    tr []
      [ td [s] [ View.completedPerc artifact ]
      , td [s] [ View.testedPerc artifact ]
      , td [s] [ View.seeArtifact settings artifact ]
      , td [s] [ partsHtml ]
      , td [s] [ partofHtml ]
      , td [s] [ pathHtml ]
      , td [s] [ locHtml ]
      , td [s] [ textHtml ]
      ]

-- given the app settings + an artifact, as well as
-- the view when expanded and the function to perform
-- on the config (config + expanded = config_expanded)
-- return a view that enables all these things
expandable : 
 Bool -> Settings -> Artifact 
  -> (Settings -> Artifact -> Html AppMsg)
  -> (ArtifactConfig -> Bool -> ArtifactConfig)
  -> Html AppMsg
expandable expanded settings artifact html setConfig = 
  if expanded then
    div []
      [ button 
        [ class "btn regular"
        , onClick (ArtifactsMsg <| SetExpand artifact.id setConfig False)
        ] [ i [ class "fa fa-minus-square-o m0" ] [] ]
      , html settings artifact
      ]
  else
    div [] 
      [ button 
        [ class "btn regular"
        , onClick (ArtifactsMsg <| SetExpand artifact.id setConfig True)
        ] [ i [ class "fa fa-plus-square-o m0" ] [] ]
      ]
