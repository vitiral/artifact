module Routing exposing (router, routerMsg)

import Navigation
import UrlParser as UP exposing ((</>))
import Messages exposing (AppMsg(..), Route(..), createUrl, editingUrl)


matchers : UP.Parser (Route -> a) a
matchers =
    UP.oneOf
        [ UP.map ArtifactsRoute UP.top
        , UP.map ArtifactNameRoute (UP.s "artifacts" </> UP.string)
        , UP.map ArtifactsRoute (UP.s "artifacts")
        , UP.map ArtifactCreateRoute (UP.s createUrl)
        , UP.map ArtifactEditingRoute (UP.s editingUrl)
        ]



-- routes a location object to it's Route


router : Navigation.Location -> Route
router location =
    case UP.parseHash matchers location of
        Just route ->
            route

        Nothing ->
            NotFoundRoute



-- convert a location into a AppMsg


routerMsg : Navigation.Location -> AppMsg
routerMsg location =
    RouteChange (router location)
