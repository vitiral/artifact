module Artifacts.Update exposing (..)

import String
import Dict
import Navigation

import Models exposing (Model)
import Messages exposing (AppMsg(AppError))
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing 
  (Artifact, ArtifactEditable, Artifacts, NameKey
  , ArtifactsResponse
  , artifactsUrl, artifactNameUrl
  , initName, indexNameUnchecked
  , artifactsFromList)

update : Msg -> Model -> ( Model, Cmd AppMsg )
update msg model =
  case msg of
    NewArtifacts newArtifacts ->
      ( { model | artifacts = artifactsFromList newArtifacts }
      , Cmd.none )

    ShowArtifacts ->
      ( model, Navigation.newUrl artifactsUrl )

    ShowArtifact name ->
      ( model
      , Navigation.newUrl 
        <| artifactNameUrl 
        <| String.toLower (indexNameUnchecked name) )

    ColumnsChanged columns ->
      let 
        s = model.state
        state = { s | columns = columns }
      in
        ( { model |  state = state }, Cmd.none )

    EditStateChanged edit ->
      let 
        s = model.state
        state = { s | edit = edit }
      in
        ( { model | state = state }, Cmd.none )

    SearchChanged search ->
      let
        s = model.state
        state = { s | search = search }
      in
        ( { model | state = state }, Cmd.none )

    ArtifactEdited name edited ->
      case Dict.get name model.artifacts of
        Just art ->
          ( { model | artifacts = setEdited model.artifacts art edited }
          , Cmd.none )
        Nothing ->
          ( model, Cmd.none ) -- TODO: should be error

-- set the edited variable on the requested artifact
setEdited : Artifacts -> Artifact -> ArtifactEditable -> Artifacts
setEdited artifacts art edited =
  Dict.insert art.name.value { art | edited = Just edited } artifacts
