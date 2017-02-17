module Main exposing (..)

import Navigation
--import Html exposing (program)
import Messages exposing (AppMsg(..), Route)
import Models exposing (Model, initialErrors, initialSettings)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll, artifactsFromStrUnsafe)

type alias Flags =
  { addr: String
  }

initialModel : String -> Route -> Model
initialModel addr route =
  -- slightly hacky, but this is how we inject the artifacts-json into
  -- the static webpage -- we just replace REPLACE_WITH_ARTIFACTS with
  -- the raw json string (with proper escapes)
  { artifacts = artifactsFromStrUnsafe "REPLACE_WITH_ARTIFACTS"
  , route = route
  , errors = initialErrors
  , settings = initialSettings
  , addr = addr
  }

init : Navigation.Location -> (Model, Cmd AppMsg)
init location =
    let
      model = initialModel "fake-addr" <| Routing.router location
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
