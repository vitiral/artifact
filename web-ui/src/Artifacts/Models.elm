module Artifacts.Models exposing (..)

import Dict
import Set
import Regex
import JsonRpc exposing (RpcError)


artifactValidRaw : String
artifactValidRaw =
    "(REQ|SPC|RSK|TST)(-[A-Z0-9_-]*[A-Z0-9_])?"


artifactValidPat : Regex.Regex
artifactValidPat =
    Regex.regex <| "^" ++ artifactValidRaw ++ "$"



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
    { raw : String
    , value : String
    }



-- How artifacts are stored


type alias Artifacts =
    Dict.Dict ArtifactId Artifact


initialArtifacts : Artifacts
initialArtifacts =
    Dict.empty



-- representation of an Artifact object


type alias Artifact =
    { id : ArtifactId
    , revision : Int
    , name : Name
    , def : String
    , text : String
    , partof : List Name
    , parts : List Name
    , code : Maybe Loc
    , done : Maybe String
    , completed : Float
    , tested : Float
    , edited : Maybe EditableArtifact
    }



-- Editable part of an artifact


type alias EditableArtifact =
    { name : String
    , def : String
    , text : String
    , partof : List Name
    , done : String
    }



-- gets the edited variable of the artifact
-- or creates the default one


getEditable : Artifact -> EditableArtifact
getEditable artifact =
    case artifact.edited of
        Just e ->
            e

        Nothing ->
            createEditable artifact


createEditable : Artifact -> EditableArtifact
createEditable artifact =
    { name = artifact.name.raw
    , def = artifact.def
    , text = artifact.text
    , partof = artifact.partof
    , done =
        case artifact.done of
            Just s ->
                s

            Nothing ->
                ""
    }


type alias ArtifactsResponse =
    { result : Maybe (List Artifact)
    , error : Maybe RpcError
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
    String.toUpper name



-- get the real name from a raw name
-- return Err if name is invalid


indexName : String -> Result String String
indexName name =
    let
        index =
            indexNameUnchecked name
    in
        if Regex.contains artifactValidPat index then
            Ok index
        else
            Err ("Invalid name: " ++ name)


initName : String -> Result String Name
initName name =
    let
        value =
            indexName name
    in
        case value of
            Ok value ->
                Ok <|
                    { raw = name
                    , value = value
                    }

            Err err ->
                Err err



-- convert a list of artifacts to a dictionary by Name


artifactsFromList : List Artifact -> Artifacts
artifactsFromList artifacts =
    let
        pairs =
            List.map (\a -> ( a.id, a )) artifacts
    in
        Dict.fromList pairs



-- VIEW Models
-- artifact attributes which can be displayed
-- or searched for


type alias Columns =
    { parts : Bool
    , partof : Bool
    , text : Bool
    , def : Bool
    , loc : Bool
    }


initialColumns : Columns
initialColumns =
    { parts = True
    , partof = False
    , text = True
    , def = False
    , loc = False
    }


type alias Search =
    { pattern :
        String

    -- the pattern to search for
    , name : Bool
    , parts : Bool
    , partof : Bool
    , text : Bool
    }


initialSearch : Search
initialSearch =
    { pattern = ""
    , name = True
    , parts = False
    , partof = False
    , text = False
    }


type alias TextViewState =
    { rendered_edit :
        Bool

    -- display the rendered tab for edit view
    , rendered_read :
        Bool

    -- display the rendered tab for read view
    }


initialTextViewState : TextViewState
initialTextViewState =
    { rendered_edit = False
    , rendered_read = True
    }
