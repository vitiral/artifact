module Artifacts.Models exposing (..)

import JsonRpc exposing (RpcError)


type alias ArtifactId =
  Int

type alias Loc =
  { path : String
  , row : Int
  , col : Int
  }

type alias Artifact =
  { id : ArtifactId
  , name : String
  , path : String
  , text : String
  , partof : List String
  , parts : List String
  , loc : Maybe Loc
  , completed : Float
  , tested : Float
  }

type alias ArtifactsResponse =
  { result: Maybe (List Artifact)
  , error: Maybe RpcError
  }

