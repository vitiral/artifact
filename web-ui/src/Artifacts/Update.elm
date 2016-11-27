module Artifacts.Update exposing (..)

import Navigation

import Http exposing (Error(..))
import Messages exposing (AppMsg(AppError))
import JsonRpc exposing (RpcError)
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing 
  (Artifact, ArtifactId, ArtifactConfig
  , ArtifactsResponse
  , artifactUrl, artifactsUrl)
import Artifacts.Commands exposing (save)

update : Msg -> List Artifact -> ( List Artifact, Cmd AppMsg )
update msg artifacts =
  case msg of
    NewArtifacts newArtifacts ->
      ( newArtifacts, Cmd.none )

    ShowArtifacts ->
      ( artifacts, Navigation.newUrl artifactsUrl )

    ShowArtifact id ->
      ( artifacts, Navigation.newUrl (artifactUrl id) )

    SetExpand id setConfig value ->
      ( setExpand artifacts id setConfig value, Cmd.none)

    --ChangeLevel id amount ->
    --  ( artifacts
    --  , Cmd.batch (changeLevelCommands id amount artifacts)
    --  )

    SaveArtifact result -> case result of
      Err err ->
        -- TODO: do something else here
        ( artifacts, Navigation.newUrl "error"  )

      Ok newArtifact ->
        ( updateArtifact newArtifact artifacts
        , Cmd.none )


--changeLevelCommands artifactId howMuch artifacts =
--  let
--    cmdForArtifact existingArtifact =
--      if existingArtifact.id == artifactId then
--        save { existingArtifact | level = existingArtifact.level + howMuch }
--      else
--        Cmd.none
--  in
--    List.map cmdForArtifact artifacts

setExpand : 
    List Artifact -> ArtifactId -> 
    (ArtifactConfig -> Bool -> ArtifactConfig)
    -> Bool -> List Artifact
setExpand artifacts id setConfig value  =
  let
    select art =
      if art.id == id then
        let
          newConfig = setConfig art.config value
        in 
          { art | config = newConfig }
      else
        art
  in
    List.map select artifacts

updateArtifact updatedArtifact artifacts =
  let
    select existingArtifact =
      if existingArtifact.id == updatedArtifact.id then
        updatedArtifact
      else
        existingArtifact
  in
    List.map select artifacts
