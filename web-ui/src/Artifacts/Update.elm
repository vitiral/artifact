module Artifacts.Update exposing (..)

import Navigation

import Http exposing (Error(..))
import Messages exposing (AppMsg(AppError))
import JsonRpc exposing (RpcError)
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing 
  (Artifact, ArtifactId, ArtifactsResponse)
import Artifacts.Commands exposing (save)

update : Msg -> List Artifact -> ( List Artifact, Cmd AppMsg )
update msg artifacts =
  case msg of
    NewArtifacts newArtifacts ->
      ( newArtifacts, Cmd.none )

    ShowArtifacts ->
      ( artifacts, Navigation.newUrl "#artifacts" )

    ShowArtifact id ->
      ( artifacts, Navigation.newUrl ("#artifacts/" ++ (toString id)) )

    ChangeLevel id amount ->
      ( artifacts
      , Cmd.batch (changeLevelCommands id amount artifacts)
      )

    SaveArtifact result -> case result of
      Err err ->
        -- TODO: do something else here
        ( artifacts, Navigation.newUrl "error"  )

      Ok newArtifact ->
        ( updateArtifact newArtifact artifacts
        , Cmd.none )


changeLevelCommands artifactId howMuch artifacts =
  let
    cmdForArtifact existingArtifact =
      if existingArtifact.id == artifactId then
        save { existingArtifact | level = existingArtifact.level + howMuch }
      else
        Cmd.none
  in
    List.map cmdForArtifact artifacts

updateArtifact updatedArtifact artifacts =
  let
    select existingArtifact =
      if existingArtifact.id == updatedArtifact.id then
        updatedArtifact
      else
        existingArtifact
  in
    List.map select artifacts
