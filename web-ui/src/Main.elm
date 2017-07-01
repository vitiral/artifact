module Main exposing (..)

import Set
import Dict
import Navigation
import Messages exposing (AppMsg(..), Route)
import Models exposing (..)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll, artifactsFromStrUnsafe)


initialModel : Navigation.Location -> Flags -> Route -> Model
initialModel location flags route =
    { artifacts = artifactsFromStrUnsafe "[]"
    , files = Set.empty
    , names = Dict.empty
    , route = route
    , location = location
    , logs = []
    , flags = flags
    , state = initialState
    , jsonId = 1
    , create = Nothing
    }


init : Flags -> Navigation.Location -> ( Model, Cmd AppMsg )
init flags location =
    let
        model =
            initialModel location flags <| Routing.router location
    in
        ( model, fetchAll model )


subscriptions : Model -> Sub AppMsg
subscriptions model =
    Sub.none



-- MAIN


main : Program Flags Model AppMsg
main =
    Navigation.programWithFlags Routing.routerMsg
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
