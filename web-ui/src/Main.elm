module Main exposing (..)

import Dict
import Navigation
import Messages exposing (AppMsg(..), Route)
import Models exposing (Model, initialLogs, initialSettings, initialState)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll, artifactsFromStrUnsafe)


type alias Flags =
    { addr : String
    }


initialModel : Navigation.Location -> String -> Route -> Model
initialModel location addr route =
    { artifacts = artifactsFromStrUnsafe "[]"
    , names = Dict.empty
    , route = route
    , location = location
    , logs = initialLogs
    , settings = initialSettings False
    , addr = addr
    , state = initialState
    , jsonId = 1
    , create = Nothing
    }


init : Flags -> Navigation.Location -> ( Model, Cmd AppMsg )
init flags location =
    let
        model =
            initialModel location flags.addr <| Routing.router location
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
