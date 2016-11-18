module Main exposing (..)

import Navigation
--import Html exposing (program)
import Messages exposing (AppMsg(..), Route)
import Models exposing (Model, initialModel)
import View exposing (view)
import Update exposing (update)
import Routing
import Artifacts.Commands exposing (fetchAll)

init : Navigation.Location -> (Model, Cmd AppMsg)
init location =
    (initialModel (Routing.router location)
    , fetchAll )


subscriptions : Model -> Sub AppMsg
subscriptions model =
  Sub.none

-- MAIN

main =
    Navigation.program Routing.routerMsg
      { init = init
      , view = view
      , update = update
      , subscriptions = subscriptions
      }
