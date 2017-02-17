module Artifacts.Update exposing (..)

import String
import Dict
import Navigation

import Messages exposing (AppMsg(AppError))
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing 
  (Artifact, ArtifactEditable, Artifacts, NameKey, ArtifactConfig
  , ArtifactsResponse
  , artifactsUrl, artifactNameUrl
  , initName, indexNameUnchecked
  , artifactsFromList)

update : Msg -> Artifacts -> ( Artifacts, Cmd AppMsg )
update msg artifacts =
  case msg of
    NewArtifacts newArtifacts ->
      ( artifactsFromList newArtifacts, Cmd.none )

    ShowArtifacts ->
      ( artifacts, Navigation.newUrl artifactsUrl )

    ShowArtifact name ->
      ( artifacts
      , Navigation.newUrl 
        <| artifactNameUrl 
        <| String.toLower (indexNameUnchecked name) )

    SetExpand name setConfig value ->
      case Dict.get name artifacts of
        Just art -> 
          ( setExpand artifacts art setConfig value, Cmd.none )
        Nothing ->
          ( artifacts, Cmd.none ) -- TODO: should be error

    ArtifactEdited name edited ->
      case Dict.get name artifacts of
        Just art ->
          ( setEdited artifacts art edited, Cmd.none )
        Nothing ->
          ( artifacts, Cmd.none ) -- TODO: should be error

-- set the edited variable on the requested artifact
setEdited : Artifacts -> Artifact -> ArtifactEditable -> Artifacts
setEdited artifacts art edited =
  Dict.insert art.name.value { art | edited = Just edited } artifacts

-- set the "expand" setting to value
setExpand : 
    Artifacts -> Artifact -> (ArtifactConfig -> Bool -> ArtifactConfig)
    -> Bool -> Artifacts
setExpand artifacts art setConfig value =
  Dict.insert art.name.value 
    { art | config = setConfig art.config value } 
    artifacts
