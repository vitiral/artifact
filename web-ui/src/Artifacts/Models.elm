module Artifacts.Models exposing (..)

import JsonRpc exposing (RpcError)


type alias ArtifactId =
  Int

type alias Artifact =
  { id : ArtifactId
  , name : String
  , level : Int
  }

new : Artifact
new =
  { id = 0
  , name = ""
  , level = 1
  }

type alias ArtifactsResponse =
  { result: Maybe (List Artifact)
  , error: Maybe RpcError
  }

