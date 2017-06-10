module Artifacts.Update exposing (..)

import String
import Dict
import Navigation

import Models exposing (Model, nameIds, getArtifact, log, logInvalidId)
import Messages exposing (AppMsg(AppError))
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing 
  (Artifact, EditableArtifact, Artifacts, NameKey
  , createEditable
  , ArtifactsResponse
  , artifactsUrl, artifactNameUrl
  , initName, indexNameUnchecked
  , artifactsFromList)
import Artifacts.Commands exposing (saveArtifacts)

update : Msg -> Model -> ( Model, Cmd AppMsg )
update msg model =
  case msg of
    ReceivedArtifacts artifactList ->
      let
        artifacts = artifactsFromList artifactList
        names = nameIds artifacts
      in
        ( handleReceived model artifactList
        , Cmd.none )

    ShowArtifacts ->
      ( model, Navigation.newUrl artifactsUrl )

    ShowArtifact name ->
      ( model
      , Navigation.newUrl 
        <| artifactNameUrl 
        <| String.toLower (indexNameUnchecked name) )

    ChangeColumns columns ->
      let 
        s = model.state
        state = { s | columns = columns }
      in
        ( { model |  state = state }, Cmd.none )

    ChangeTextViewState textView ->
      let 
        s = model.state
        state = { s | textView = textView }
      in
        ( { model | state = state }, Cmd.none )

    ChangeSearch search ->
      let
        s = model.state
        state = { s | search = search }
      in
        ( { model | state = state }, Cmd.none )

    EditArtifact id edited ->
      case Dict.get id model.artifacts of
        Just art ->
          ( { model | artifacts = setEdited model.artifacts art (Just edited) }
          , Cmd.none )
        Nothing ->
          ( logInvalidId model "edit" id, Cmd.none )

    CancelEditArtifact id ->
      case Dict.get id model.artifacts of
        Just art ->
          ( { model | artifacts = setEdited model.artifacts art Nothing }
          , Cmd.none )
        Nothing ->
          ( logInvalidId model "cancel" id, Cmd.none )

    SaveArtifact id ->
      case Dict.get id model.artifacts of
        Just art ->
          let
            model2 = log model <| "trying to save " ++ (toString id)
            model3 = { model2 | jsonId = model2.jsonId + 1 }
          in
            ( model3, saveArtifacts model [ art ])
        Nothing ->
          ( logInvalidId model "save" id, Cmd.none )

-- set the edited variable on the requested artifact
setEdited : Artifacts -> Artifact -> Maybe EditableArtifact -> Artifacts
setEdited artifacts art edited =
  Dict.insert art.id { art | edited = edited } artifacts

-- we need to make sure we keep any edited data that has not been applied but
handleReceived : Model -> List Artifact -> Model
handleReceived model artifactList =
  let
    keepEdited : Artifact -> Artifact
    keepEdited newArt =
      case Dict.get newArt.id model.artifacts of
        Just oldArt -> 
          let
            -- get the edited, keeping in mind that changes may
            -- have been applied
            edited = if oldArt.revision == newArt.revision then
              oldArt.edited
            else
              case oldArt.edited of
                Just e ->
                  if e == createEditable newArt then
                    Nothing -- the changes were applied
                  else
                    -- The changes have not been applied
                    -- but the artifact has changed (by someone else)!
                    -- That's fine, just keep the old edit data
                    -- TODO: log or something here. The UI should show
                    -- "this artifact has changed" or something.
                    Just e 
                Nothing ->
                  Nothing
          in
            -- return the new one, but keep the edited data
            { newArt | edited = edited }
        Nothing ->
          -- artifact is completely new
          newArt

    processed = List.map keepEdited artifactList
    artifacts = artifactsFromList processed
    names = nameIds artifacts
  in
    { model | artifacts = artifacts, names = names }


