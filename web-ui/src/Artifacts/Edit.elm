module Artifacts.Edit exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, value, href)
import Html.Events exposing (onClick)

import Models exposing (Settings)
import Artifacts.Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)

view : Settings -> Artifact -> Html AppMsg
view settings model =
  div []
    [ nav model
    , form settings model
    ]

nav : Artifact -> Html AppMsg
nav artifact =
  div [ class "clearfix mb2 white bg-black p1" ]
    [ listBtn ]


form : Settings -> Artifact -> Html AppMsg
form settings artifact =
  div [ class "m3" ]
    [ h1 [] [ text artifact.name ]
    , formLevel settings artifact
    ]

formLevel : Settings -> Artifact -> Html AppMsg
formLevel settings artifact =
  div
    [ class "clearfix py1" ]
    [ div [ class "col col-5" ] [ text "Level" ]
    , div [ class "col col-7" ]
      [ span [ class "h2 bold" ] [ text (toString artifact.level) ]
      , btnLevelDecrease artifact
      , btnLevelIncrease artifact
      ]
    ]

btnLevelDecrease : Artifact -> Html AppMsg 
btnLevelDecrease artifact =
  a [ class "btn ml1 h1", onClick (ArtifactsMsg <| ChangeLevel artifact.id -1) ]
    [ i [ class "fa fa-minus-circle" ] [] ]

btnLevelIncrease : Artifact -> Html AppMsg 
btnLevelIncrease artifact =
  a [ class "btn ml1 h1", onClick (ArtifactsMsg <| ChangeLevel artifact.id 1) ]
    [ i [ class "fa fa-plus-circle" ] [] ]

listBtn : Html AppMsg
listBtn =
  button
    [ class "btn regular"
    , onClick (ArtifactsMsg ShowArtifacts)
    ]
    [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]
