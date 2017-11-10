module View exposing (..)

import Html exposing (..)
import Markdown exposing (toHtml)
import Messages exposing (AppMsg(..), Route(..))
import Models exposing (..)
import Artifacts.Models exposing (..)
import Artifacts.List
import Artifacts.Edit
import Artifacts.View
import Artifacts.ViewRendered
import Artifacts.Nav as Nav
import Help


view : Model -> Html AppMsg
view model =
    div []
        [ page model ]


page : Model -> Html AppMsg
page model =
    case model.route of
        ArtifactsRoute ->
            Artifacts.List.view model model.artifacts

        ArtifactNameRoute raw_name ->
            case indexName raw_name of
                Ok name ->
                    case getArtifact name model of
                        Just artifact ->
                            case getEditViewOption model artifact of
                                ReadChoice choice ->
                                    Artifacts.ViewRendered.view model choice

                                EditChoice choice ->
                                    Artifacts.Edit.view model choice

                        Nothing ->
                            notFoundView

                Err error ->
                    div []
                        [ text <| "invalid artifact name: " ++ error
                        ]

        ArtifactEditingRoute ->
            Artifacts.View.viewEditing model

        ArtifactCreateRoute ->
            let
                choice = CreateChoice (getCreateArtifact model)
            in
                Artifacts.Edit.view model choice

        CheckRoute ->
            viewCheck model

        HelpRoute route ->
            case Help.getPage route of
                Just h ->
                    div [] [ Nav.bar model <| Nav.helpBar, Help.viewPage h ]

                Nothing ->
                    text <| "Help page " ++ route ++ " not found."

        NotFoundRoute ->
            notFoundView


notFoundView : Html a
notFoundView =
    div []
        [ text "Artifact Name Not Found"
        ]


viewCheck : Model -> Html AppMsg
viewCheck model =
    div []
        [ Nav.bar model Nav.helpBar
        , h1 [] [ text "Failed Checks" ]
        , toHtml [] model.checked
        ]
