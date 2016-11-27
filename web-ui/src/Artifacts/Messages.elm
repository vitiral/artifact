module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models exposing (ArtifactId, Artifact, ArtifactConfig)


type Msg
  = NewArtifacts (List Artifact)
  | ShowArtifacts
  | ShowArtifact ArtifactId
  | SetExpand ArtifactId (ArtifactConfig -> Bool -> ArtifactConfig) Bool
  | SaveArtifact (Result Http.Error Artifact)
