module View exposing (..)

import Html exposing (Html, div, text)
import Messages exposing (AppMsg(..), Route(..))
import Models exposing (Model)
import Artifacts.List
import Artifacts.Edit
import Artifacts.Models exposing (ArtifactId, indexNameUnchecked)

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
        id_maybe = List.filter (\a -> a.name.value == name) model.artifacts
      in
        case List.head id_maybe of
          Just artifact ->
            artifactEditPage model artifact.id
          Nothing -> 
            notFoundView

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
          Artifacts.Edit.view model artifact

        Nothing ->
          notFoundView

notFoundView : Html a
notFoundView =
  div []
    [ text "Artifact Name Not Found"
    ]

