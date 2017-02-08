module View exposing (..)

import Dict

import Html exposing (Html, div, text)
import Messages exposing (AppMsg(..), Route(..))
import Models exposing (Model)
import Artifacts.List
import Artifacts.Edit
import Artifacts.Models exposing (NameKey, indexNameUnchecked)

-- partof: #SPC-web-view
view : Model -> Html AppMsg
view model =
  div []
    [ page model ]

page : Model -> Html AppMsg
page model =
  case model.route of
    ArtifactsRoute ->
      Artifacts.List.view model model.artifacts

    ArtifactNameRoute raw_name ->
      let
        -- TODO: should fail if invalid name
        name = indexNameUnchecked raw_name
      in
        if Dict.member name model.artifacts then
          artifactEditPage model name
        else
          notFoundView

    NotFoundRoute ->
      notFoundView

artifactEditPage : Model -> NameKey -> Html AppMsg
artifactEditPage model name =
  case Dict.get name model.artifacts of
    Just artifact -> 
      Artifacts.Edit.view model artifact

    Nothing ->
      notFoundView

notFoundView : Html a
notFoundView =
  div []
    [ text "Artifact Name Not Found"
    ]

