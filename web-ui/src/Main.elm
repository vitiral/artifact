module Main exposing (..)

import Navigation
--import Html exposing (program)
import Messages exposing (AppMsg(..), Route)
import Models exposing (Model, initialModel)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll)

type alias Flags =
  { addr: String
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
