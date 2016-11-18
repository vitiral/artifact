module View exposing (..)

import Html exposing (Html, div, text)
import Messages exposing (AppMsg(..), Route(..))
import Models exposing (Model)
import Artifacts.List
import Artifacts.Edit
import Artifacts.Models exposing (ArtifactId)


view : Model -> Html AppMsg
view model =
  div []
    [ page model ]


page : Model -> Html AppMsg
page model =
  case model.route of
    ArtifactsRoute ->
      Artifacts.List.view model.settings model.artifacts

    ArtifactRoute id ->
      artifactEditPage model id

    NotFoundRoute ->
      notFoundView

artifactEditPage : Model -> ArtifactId -> Html AppMsg
artifactEditPage model id =
  let
    maybeArtifact =
      model.artifacts
        |> List.filter (\artifact -> artifact.id == id)
        |> List.head
  in
      case maybeArtifact of
        Just artifact ->
          Artifacts.Edit.view model.settings artifact

        Nothing ->
          notFoundView

notFoundView : Html a
notFoundView =
  div []
    [ text "Route Not Found"
    ]

