module Artifacts.Messages exposing (..)

import Artifacts.Models exposing (..)


type Msg
    = ReceivedArtifacts (List Artifact)
    | ShowArtifacts
    | ShowArtifact String
    | ShowEditing
    | CreateArtifact
    | ChangeColumns Columns
    | ChangeSearch Search
    | ChangeTextViewState TextViewState
    | EditArtifact EditOption
    | CancelEditArtifact EditOption
    | SaveArtifact EditOption
    | DeleteArtifact Artifact
