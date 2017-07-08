module View exposing (..)

import Html exposing (..)
import Markdown exposing (toHtml)
import Messages exposing (AppMsg(..), Route(..))
import Models exposing (..)
import Artifacts.Models exposing (..)
import Artifacts.List
import Artifacts.Edit
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
                            Artifacts.Edit.view model <| getOption model artifact

                        Nothing ->
                            notFoundView

                Err error ->
                    div []
                        [ text <| "invalid artifact name: " ++ error
                        ]

        ArtifactEditingRoute ->
            Artifacts.Edit.viewEditing model

        ArtifactCreateRoute ->
            getCreateArtifact model
                |> CreateChoice
                |> EditChoice
                |> Artifacts.Edit.view model

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


{-| get the viewing option for an existing artifact
-}
getOption : Model -> Artifact -> ViewOption
getOption model artifact =
    if model.flags.readonly then
        ReadChoice artifact
    else
        case artifact.edited of
            Just e ->
                EditChoice <| ChangeChoice artifact e

            Nothing ->
                ReadChoice <| artifact


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
