module Artifacts.Models exposing (..)

import Regex

import JsonRpc exposing (RpcError)


spacePat : Regex.Regex
spacePat = Regex.regex " "

artifactValidPat : Regex.Regex
artifactValidPat = Regex.regex "^(REQ|SPC|RSK|TST)(-[A-Z0-9_-]*[A-Z0-9_])?$"


type alias ArtifactId =
  Int

type alias Loc =
  { path : String
  , row : Int
  , col : Int
  }

type alias Text =
  { raw: String
  , value: String
  }

type alias Name =
  { raw: String
  , value: String
  }

type alias Artifact =
  { id : ArtifactId
  , name : Name
  , path : String
  , text : Text
  , partof : List Name
  , parts : List Name
  , loc : Maybe Loc
  , completed : Float
  , tested : Float
  , config: ArtifactConfig
  }

type alias ArtifactConfig =
  { partsExpanded : Bool
  , partofExpanded : Bool
  , pathExpanded : Bool
  , locExpanded : Bool
  , textExpanded : Bool
  }

type alias ArtifactsResponse =
  { result: Maybe (List Artifact)
  , error: Maybe RpcError
  }

defaultConfig : ArtifactConfig
defaultConfig =
  { partsExpanded = False
  , partofExpanded = False
  , pathExpanded = False
  , locExpanded = False
  , textExpanded = False
  }


artifactsUrl : String
artifactsUrl =
  "#artifacts" 

--artifactUrl : ArtifactId -> String
--artifactUrl id =
--  "#artifacts/" ++ (toString id)

artifactNameUrl : String -> String
artifactNameUrl name =
  "#artifacts/" ++ name


-- get the real name from a raw name
indexNameUnchecked : String -> String
indexNameUnchecked name =
  let
    replaced = Regex.replace Regex.All spacePat (\_ -> "") name
  in
    String.toUpper replaced


-- get the real name from a raw name
-- return Err if name is invalid
-- parof: #SPC-web-validation
indexName : String -> Result String String
indexName name =
  let
    index = indexNameUnchecked name
  in
    if Regex.contains artifactValidPat index then
      Ok index
    else
      Err ("Invalid name: " ++ name)

initName : String -> Result String Name
initName name =
  let
    value = indexName name
  in
    case value of
      Ok value -> Ok <|
        { raw = name
        , value = value
        }
      Err err ->
        Err err
