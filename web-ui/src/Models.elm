module Models exposing (..)

import Set
import Dict
import Navigation
import Messages exposing (Route)
import Artifacts.Models exposing (..)
import Utils exposing (isJust)


-- CONSTANTS
-- TYPES


{-| user given flags
-}
type alias Flags =
    { readonly : Bool
    , path_url : String
    }


type alias Model =
    { artifacts : Artifacts
    , files : Set.Set String
    , names :
        Dict.Dict NameKey ArtifactId
    , route : Route
    , location : Navigation.Location
    , logs : List LogMsg
    , flags : Flags
    , state : State
    , jsonId : Int
    , create : Maybe EditableArtifact
    }


{-| current user selections. TODO: store this in a cookie or something...
-}
type alias State =
    { columns : Columns
    , search : Search
    , textView : TextViewState
    }


{-| We can log either a success or a failure to the user
-}
type LogMsg
    = LogOk String
    | LogErr String



-- INIT


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
        Just c ->
            c

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
getFiles : Model -> Maybe String -> List String
getFiles model current =
    let
        files =
            case current of
                Just c ->
                    Set.insert c model.files

                Nothing ->
                    model.files
    in
        Set.remove "PARENT" files
            |> Set.toList
            |> List.sort
