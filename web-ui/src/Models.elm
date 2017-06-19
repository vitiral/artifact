module Models exposing (..)

import Dict
import Navigation
import Messages exposing (Route)
import Artifacts.Models
    exposing
        ( Artifact
        , Artifacts
        , ArtifactId
        , NameKey
        , Columns
        , TextViewState
        , Search
        , initialColumns
        , initialTextViewState
        , initialSearch
        )
import Utils exposing (isJust)


-- MODEL: application level model, holds all app data


type alias Model =
    { artifacts : Artifacts
    , names :
        Dict.Dict NameKey ArtifactId

    -- get the id of a name
    , route : Route
    , location : Navigation.Location
    , logs : Logs
    , settings : Settings
    , addr : String
    , state : State
    , jsonId : Int
    }


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



-- settings from cmdline tool (injected into js)


type alias Settings =
    { readonly : Bool
    }


initialSettings : Bool -> Settings
initialSettings readonly =
    { readonly = readonly
    }



-- ERRORS: place to store errors that happen
-- TODO: this is supposed to display a list
-- that disappears over time


type alias Logs =
    { all : List String
    }


initialLogs : Logs
initialLogs =
    { all = []
    }



-- log an error


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



-- STATE
-- current user selections
-- TODO: store this in a cookie or something...


type alias State =
    { columns : Columns
    , search : Search
    , textView : TextViewState
    }


initialState : State
initialState =
    { columns = initialColumns
    , search = initialSearch
    , textView = initialTextViewState
    }
