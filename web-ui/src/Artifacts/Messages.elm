module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models exposing (
  NameKey, Artifact, Artifacts, ArtifactEditable, Columns, Search)


type Msg
  = NewArtifacts (List Artifact)
  | ShowArtifacts
  | ShowArtifact String
  | ColumnsChanged Columns 
  | SearchChanged Search
  | ArtifactEdited NameKey ArtifactEditable
