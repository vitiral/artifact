module Artifacts.Models exposing (..)
import Dict
import Set

import Regex

import JsonRpc exposing (RpcError)


spacePat : Regex.Regex
spacePat = Regex.regex " "

artifactValidRaw : String
artifactValidRaw = 
  "(REQ|SPC|RSK|TST)(-[A-Z0-9_-]*[A-Z0-9_])?"

artifactValidPat : Regex.Regex
artifactValidPat = Regex.regex <| "^" ++ artifactValidRaw ++ "$"

-- pretty much only used when updating artifacts
type alias ArtifactId =
  Int

-- the standard lookup method for artifacts
type alias NameKey = 
  String

type alias Loc =
  { path : String
  , line : Int
  }

type alias Name =
  { raw: String
  , value: String
  }

-- How artifacts are stored
type alias Artifacts = Dict.Dict NameKey Artifact

initialArtifacts : Artifacts
initialArtifacts =
  Dict.empty


-- representation of an Artifact object
type alias Artifact =
  { id : ArtifactId
  , name : Name
  , path : String
  , text : String
  , partof : List Name
  , parts : List Name
  , code : Maybe Loc
  , done : Maybe String
  , completed : Float
  , tested : Float
  , edited : Maybe ArtifactEditable
  }

-- Editable part of an artifact
type alias ArtifactEditable =
  { name : Name
  , path : String
  , text : String
  , partof : List Name
  }

-- gets the edited variable of the artifact
-- or creates the default one
getEdited : Artifact -> ArtifactEditable
getEdited artifact =
  case artifact.edited of
    Just e -> e
    Nothing ->
      { name = artifact.name
      , path = artifact.path
      , text = artifact.text
      , partof = artifact.partof
      }


type alias ArtifactsResponse =
  { result: Maybe (List Artifact)
  , error: Maybe RpcError
  }


artifactsUrl : String
artifactsUrl =
  "#artifacts" 

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

-- convert a list of artifacts to a dictionary
artifactsFromList : List Artifact -> Artifacts
artifactsFromList artifacts =
  let
    pairs = List.map (\a -> ( a.name.value, a )) artifacts
  in
    Dict.fromList pairs


-- VIEW Models

-- artifact attributes which can be displayed
-- or searched for
type alias Columns =
  { parts : Bool
  , partof : Bool
  , text : Bool
  , path : Bool
  , loc : Bool
  }

initialColumns : Columns
initialColumns =
  { parts = True
  , partof = False
  , text = True
  , path = False
  , loc = False
  }

type alias Search =
  { pattern : String  -- the pattern to search for
  , regex : Bool      -- whether to use regex or raw-string
  , name : Bool    
  , parts : Bool
  , partof : Bool
  , text : Bool
  }

initialSearch : Search
initialSearch =
  { pattern = ""
  , regex = False
  , name = True
  , parts = False
  , partof = False
  , text = False
  }

type alias EditState =
  { rendered : Bool -- display the rendered tab
  }

initialEditState : EditState
initialEditState =
  { rendered = True
  }
