module Models exposing (..)

import Set
import Dict
import Navigation
import Messages exposing (Route)
import Artifacts.Models exposing (..)
import Utils exposing (isJust)


-- TYPES


type alias Model =
    { artifacts : Artifacts
    , names :
        Dict.Dict NameKey ArtifactId
    , route : Route
    , location : Navigation.Location
    , logs : Logs
    , settings : Settings
    , addr : String
    , state : State
    , jsonId : Int
    , create : Maybe EditableArtifact
    }


type alias Settings =
    { readonly : Bool
    }


type alias Logs =
    { all : List String
    }


{-| current user selections. TODO: store this in a cookie or something...
-}
type alias State =
    { columns : Columns
    , search : Search
    , textView : TextViewState
    }



-- INIT


{-| settings from cmdline tool (injected into js)
-}
initialSettings : Bool -> Settings
initialSettings readonly =
    { readonly = readonly
    }


initialLogs : Logs
initialLogs =
    { all = []
    }


initialState : State
initialState =
    { columns = initialColumns
    , search = initialSearch
    , textView = initialTextViewState
    }



-- METHODS


nameIds : Artifacts -> Dict.Dict NameKey ArtifactId
nameIds artifacts =
    let
        pairs =
            List.map (\a -> ( a.name.value, a.id )) (Dict.values artifacts)
    in
        Dict.fromList pairs


getArtifact : NameKey -> Model -> Maybe Artifact
getArtifact name model =
    let
        id =
            Dict.get name model.names
    in
        case id of
            Just id ->
                Dict.get id model.artifacts

            Nothing ->
                Nothing


memberArtifact : NameKey -> Model -> Bool
memberArtifact name model =
    isJust (getArtifact name model)


getCreateArtifact : Model -> EditableArtifact
getCreateArtifact model =
    case model.create of
        Just c -> c
        Nothing -> 
            { name = ""
            , def = ""
            , text = ""
            , partof = []
            , done = ""
            , revision = 0
            }


{-| get all definition file paths
-}
getDefs : Model -> List String
getDefs model =
    let
        defs =
            Set.fromList (List.map (\a -> a.def) (Dict.values model.artifacts))
    in
        List.sort <| Set.toList defs


{-| log an error
-}
log : Model -> String -> Model
log model msg =
    let
        _ =
            Debug.log msg

        logs =
            model.logs

        newLogs =
            { logs | all = List.append logs.all [ msg ] }
    in
        { model | logs = newLogs }


logInvalidId : Model -> String -> Int -> Model
logInvalidId model desc id =
    log model <| desc ++ ": invalid id " ++ (toString id)
