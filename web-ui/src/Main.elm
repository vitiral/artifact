module Main exposing (..)

import Navigation
--import Html exposing (program)
import Messages exposing (AppMsg(..), Route)
import Models exposing (Model, initialErrors, initialSettings, initialState)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll, artifactsFromStrUnsafe)

type alias Flags =
  { addr: String
  }

initialModel : String -> Route -> Model
initialModel addr route =
  { artifacts = artifactsFromStrUnsafe "[]"
  , route = route
  , errors = initialErrors
  , settings = initialSettings
  , addr = addr
  , state = initialState
  }

init : Flags -> Navigation.Location -> (Model, Cmd AppMsg)
init flags location =
    let
      model = initialModel flags.addr <| Routing.router location
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
