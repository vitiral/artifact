module Artifacts.Messages exposing (..)

import Http
import Artifacts.Models
    exposing
        ( ArtifactId
        , NameKey
        , Artifact
        , Artifacts
        , EditableArtifact
        , Columns
        , TextViewState
        , Search
        )


type Msg
    = ReceivedArtifacts (List Artifact)
    | ShowArtifacts
    | ShowArtifact String
    | ChangeColumns Columns
    | ChangeSearch Search
    | ChangeTextViewState TextViewState
    | EditArtifact ArtifactId EditableArtifact
    | CancelEditArtifact ArtifactId
    | SaveArtifact ArtifactId
