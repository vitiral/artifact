module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models exposing (ArtifactId, Artifact, ArtifactConfig)


type Msg
  = NewArtifacts (List Artifact)
  | ShowArtifacts
  | ShowArtifact String
  | SetExpand ArtifactId (ArtifactConfig -> Bool -> ArtifactConfig) Bool
  | SaveArtifact (Result Http.Error Artifact)
