module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models exposing (
  NameKey, Artifact, Artifacts, ArtifactEditable, 
  Columns, EditState, Search)


type Msg
  = NewArtifacts (List Artifact)
  | ShowArtifacts
  | ShowArtifact String
  | ColumnsChanged Columns 
  | SearchChanged Search
  | EditStateChanged EditState
  | ArtifactEdited NameKey ArtifactEditable
