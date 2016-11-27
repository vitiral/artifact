module Artifacts.Edit exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, style, value, href, readonly, rows, cols)
import Html.Events exposing (onClick)

import Models exposing (Settings)
import Artifacts.Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)
import Artifacts.View as View

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
    , div [ class "clearfix py1" ]
      [ formColumnOne settings artifact
      , formColumnTwo settings artifact
      ]
    ]

formColumnOne settings artifact =
  div [ class "col col-6" ]
    [ View.completion artifact
    , View.defined settings artifact
    , View.implemented settings artifact
    , div [ class "clearfix py1" ] 
      [
      div [ class "col col-6" ] 
        [ h3 [] [ text "Parts" ]
        , View.parts settings artifact
        ]
      , div [ class "col col-6" ] 
        [ h3 [] [ text "Partof" ]
        , View.partof settings artifact
        ]
      ]
    ]


formColumnTwo settings artifact =
  div [ class "col col-6" ] 
    [ h3 [] [ text "Text" ]
    , textarea 
      [ class "h3" -- class=h3 otherwise it is really tiny for some reason
      , rows 35, cols 80, readonly settings.readonly ] 
      [ text artifact.text ]
    ]


listBtn : Html AppMsg
listBtn =
  button
    [ class "btn regular"
    , onClick (ArtifactsMsg ShowArtifacts)
    ]
    [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]
