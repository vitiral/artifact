module Artifacts.Edit exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, style, value, href, readonly, rows, cols, id)
import Html.Events exposing (onClick)

import Models exposing (Model)
import Artifacts.Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)
import Artifacts.View as View

view : Model -> Artifact -> Html AppMsg
view model artifact =
  div []
    [ nav model
    , form model artifact
    ]

nav : Model -> Html AppMsg
nav model =
  div [ class "clearfix mb2 white bg-black p1" ]
    [ listBtn ]


form : Model -> Artifact -> Html AppMsg
form model artifact =
  div [ class "m3" ]
    [ h1 [id "ehead"] [ text artifact.name.raw ]
    , div [ class "clearfix py1" ]
      [ formColumnOne model artifact
      , formColumnTwo model artifact
      ]
    ]

formColumnOne : Model -> Artifact -> Html AppMsg
formColumnOne model artifact =
  div [ class "col col-6" ]
    [ View.completion artifact
    , View.defined model artifact
    , View.implemented model artifact
    , div [ class "clearfix py1" ] 
      [ div [ class "col col-6" ] 
        [ h3 [] [ text "Partof" ]
        , View.partof model artifact
        ]
      , div [ class "col col-6" ] 
        [ h3 [] [ text "Parts" ]
        , View.parts model artifact
        ]
      ]
    ]


formColumnTwo : Model -> Artifact -> Html AppMsg
formColumnTwo model artifact =
  div [ class "col col-6" ] 
    [ h3 [] [ text "Text" ]
    , textarea 
      [ class "h3" -- class=h3 otherwise it is really tiny for some reason
      , rows 35, cols 80, readonly model.settings.readonly 
      , id ("text_" ++ artifact.name.value)
      ] 
      [ text artifact.text ]
    ]


listBtn : Html AppMsg
listBtn =
  button
    [ class "btn regular"
    , onClick (ArtifactsMsg ShowArtifacts)
    ]
    [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]
