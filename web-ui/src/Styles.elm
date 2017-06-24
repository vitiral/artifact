module Styles exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, title)


warningStyle : String
warningStyle =
    " bold silver bg-orange p1"


warning : String -> Html m
warning msg =
    span
        [ class warningStyle, title "warning" ]
        [ i [ class "fa fa-exclamation mr1" ] []
        , text msg
        , i [ class "fa fa-exclamation ml1" ] []
        ]
