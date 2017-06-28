module Main exposing (..)

import Navigation


--import Html exposing (program)

import Messages exposing (AppMsg(..), Route)
import Models exposing (Model, initialLogs, initialSettings, initialState, nameIds)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll, artifactsFromStrUnsafe)


type alias Flags =
    { addr : String
    }


initialModel : Navigation.Location -> String -> Route -> Model
initialModel location addr route =
    -- slightly hacky, but this is how we inject the artifacts-json into
    -- the static webpage -- we just replace REPLACE_WITH_ARTIFACTS with
    -- the raw json string (with proper escapes)
    let
        artifacts =
            artifactsFromStrUnsafe "REPLACE_WITH_ARTIFACTS"
    in
        { artifacts = artifacts
        , names = nameIds artifacts
        , route = route
        , location = location
        , logs = initialLogs
        , settings = initialSettings True
        , addr = addr
        , state = initialState
        , jsonId = 1
        , create = Nothing
        }


init : Navigation.Location -> ( Model, Cmd AppMsg )
init location =
    let
        _ =
            Debug.log
                ("origin="
                    ++ location.origin
                    ++ " host="
                    ++ location.host
                    ++ " pathname="
                    ++ location.pathname
                )

        model =
            initialModel location "fake-addr" <| Routing.router location
    in
        ( model, Cmd.none )


subscriptions : Model -> Sub AppMsg
subscriptions model =
    Sub.none



-- MAIN


main : Program Never Model AppMsg
main =
    Navigation.program Routing.routerMsg
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
