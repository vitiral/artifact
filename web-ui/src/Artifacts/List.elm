module Artifacts.List exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import Messages exposing (AppMsg(..))
import Models exposing (Settings)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (Artifact)

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
    div [ class "p2" ]
      [ table []
        [ thead [] -- thead == table-header
          [ tr []  -- table row
            [ th [] [ text "Id" ] -- table header
            , th [] [ text "Name" ]
            , th [] [ text "Actions" ]
            ]
          ]
        , tbody [] (List.map2 artifactRow settings_list artifacts)
        ]
      ]

artifactRow : Settings -> Artifact -> Html AppMsg
artifactRow settings artifact =
  tr []
    [ td [] [ text (toString artifact.id) ]
    , td [] [ text artifact.name ]
    , td []
      [ seeBtn settings artifact ] 
    ]

seeBtn : Settings -> Artifact -> Html AppMsg
seeBtn settings artifact =
  button 
    [ class "btn regular"
    , onClick (ArtifactsMsg <| ShowArtifact artifact.id)
    ]
    [ i [ class <| if settings.readonly then "fa fa-eye mr1" else "fa fa-pencil mr1" ] [], text "Edit" ]
