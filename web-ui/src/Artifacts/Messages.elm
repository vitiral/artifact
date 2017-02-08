module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models exposing (
  NameKey, Artifact, Artifacts, ArtifactEditable, ArtifactConfig)


type Msg
  = NewArtifacts (List Artifact)
  | ShowArtifacts
  | ShowArtifact String
  | SetExpand NameKey (ArtifactConfig -> Bool -> ArtifactConfig) Bool
  | ArtifactEdited NameKey ArtifactEditable
