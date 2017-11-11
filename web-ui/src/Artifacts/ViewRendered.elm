module Artifacts.ViewRendered exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Models exposing (Model, getArtifact, memberArtifact, getCreateArtifact)
import Messages exposing (AppMsg(..), Route(..), HelpPage(..))
import Messages exposing (createUrl, AppMsg(..), HelpPage(..), Route(..))
import Artifacts.Models exposing (..)
import Artifacts.Nav as Nav
import Artifacts.Select as Select
import Artifacts.View as View


{-| the entire view
-}
view : Model -> Artifact -> Html AppMsg
view model artifact =
    let
        option =
            ReadChoice artifact

        nav =
            Nav.bar model <| Nav.editBar model option
    in
        div [ View.viewIdAttr option ] <|
            List.concat
                [ [ nav ]
                , View.revisionWarnings model option
                , nameHeader artifact
                , [ View.displayRenderedFamily model option ]
                , artifactInfo model artifact
                , [ View.displayRenderedText model option ]
                ]


nameHeader : Artifact -> List (Html AppMsg)
nameHeader artifact =
    let
        name_id =
            View.idAttr "name" <| ReadChoice artifact
    in
        [ h1 [ name_id ]
            [ Nav.helpBtn HelpName False
            , text artifact.name.raw
            ]
        ]


artifactInfo : Model -> Artifact -> List (Html AppMsg)
artifactInfo model artifact =
    let
        col1 =
            div [ class "col col-2" ]
                [ div [] <| View.viewCompletedPerc artifact
                , div [] <| View.viewTestedPerc artifact
                ]

        col2 =
            div [ class "col col-10" ]
                [ Select.defined model <| ReadChoice artifact
                , View.implemented model artifact
                ]
    in
        [ div [ class "clearfix py1" ] [ col1, col2 ] ]
