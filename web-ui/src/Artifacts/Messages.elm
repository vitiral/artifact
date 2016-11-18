module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models exposing (ArtifactId, Artifact)


type Msg
  = NewArtifacts (List Artifact)
  | ShowArtifacts
  | ShowArtifact ArtifactId
  | ChangeLevel ArtifactId Int
  | SaveArtifact (Result Http.Error Artifact)


