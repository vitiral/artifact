module Artifacts.Update exposing (..)

import String
import Dict
import Navigation
import Models exposing (Model, nameIds, getArtifact, log, logInvalidId)
import Messages exposing (AppMsg(AppError), Route(ArtifactNameRoute))
import Utils exposing (assertOr)
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models
    exposing
        ( Artifact
        , EditableArtifact
        , Artifacts
        , NameKey
        , createEditable
        , ArtifactsResponse
        , artifactsUrl
        , artifactNameUrl
        , initName
        , indexNameUnchecked
        , artifactsFromList
        )
import Artifacts.Commands exposing (saveArtifacts)


update : Msg -> Model -> ( Model, Cmd AppMsg )
update msg model =
    case msg of
        ReceivedArtifacts artifactList ->
            handleReceived model artifactList

        ShowArtifacts ->
            ( model, Navigation.newUrl artifactsUrl )

        ShowArtifact name ->
            ( model
            , Navigation.newUrl <|
                artifactNameUrl (indexNameUnchecked name)
            )

        ChangeColumns columns ->
            let
                s =
                    model.state

                state =
                    { s | columns = columns }
            in
                ( { model | state = state }, Cmd.none )

        ChangeTextViewState textView ->
            let
                s =
                    model.state

                state =
                    { s | textView = textView }
            in
                ( { model | state = state }, Cmd.none )

        ChangeSearch search ->
            let
                s =
                    model.state

                state =
                    { s | search = search }
            in
                ( { model | state = state }, Cmd.none )

        EditArtifact id edited ->
            case Dict.get id model.artifacts of
                Just art ->
                    let
                        -- update revision so that any warnings of prior
                        -- change go away
                        e =
                            { edited | revision = art.revision }
                    in
                        ( { model | artifacts = setEdited model.artifacts art (Just e) }
                        , Cmd.none
                        )

                Nothing ->
                    ( logInvalidId model "edit" id, Cmd.none )

        CancelEditArtifact id ->
            case Dict.get id model.artifacts of
                Just art ->
                    ( { model | artifacts = setEdited model.artifacts art Nothing }
                    , Cmd.none
                    )

                Nothing ->
                    ( logInvalidId model "cancel" id, Cmd.none )

        SaveArtifact id ->
            case Dict.get id model.artifacts of
                Just art ->
                    let
                        model2 =
                            log model <| "trying to save " ++ (toString id)

                        model3 =
                            { model2 | jsonId = model2.jsonId + 1 }
                    in
                        ( model3, saveArtifacts model [ art ] )

                Nothing ->
                    ( logInvalidId model "save" id, Cmd.none )


{-| set the edited variable on the requested artifact
-}
setEdited : Artifacts -> Artifact -> Maybe EditableArtifact -> Artifacts
setEdited artifacts art edited =
    Dict.insert art.id { art | edited = edited } artifacts


{-| we need to make sure we keep any edited data that has not been applied
-}
handleReceived : Model -> List Artifact -> ( Model, Cmd AppMsg )
handleReceived model artifactList =
    let
        process : Artifact -> ( Maybe String, Artifact )
        process newArt =
            case Dict.get newArt.id model.artifacts of
                Just oldArt ->
                    let
                        -- handle the "edited" field
                        edited =
                            handleEditedReceived oldArt newArt

                        route =
                            case model.route of
                                ArtifactNameRoute route_name ->
                                    if (indexNameUnchecked route_name) == oldArt.name.value then
                                        -- artifact name we are viewing got changed
                                        -- go to new route
                                        Just newArt.name.value
                                    else
                                        Nothing

                                _ ->
                                    Nothing
                    in
                        ( route, { newArt | edited = edited } )

                Nothing ->
                    -- artifact is completely new
                    ( Nothing, newArt )

        processed =
            List.map process artifactList

        artifacts =
            artifactsFromList <| List.map Tuple.second processed

        names =
            nameIds artifacts

        routes =
            List.filterMap Tuple.first processed

        _ =
            assertOr ((List.length routes) <= 1) 0 "impossible routes"

        ( route, cmd ) =
            case List.head routes of
                Just r ->
                    ( ArtifactNameRoute r
                    , Navigation.newUrl <| artifactNameUrl r
                    )

                Nothing ->
                    ( model.route, Cmd.none )
    in
        ( { model | artifacts = artifacts, names = names, route = route }
        , cmd
        )


{-| get the edited, keeping in mind that changes may have been applied
-}
handleEditedReceived : Artifact -> Artifact -> Maybe EditableArtifact
handleEditedReceived oldArt newArt =
    if oldArt.revision == newArt.revision then
        oldArt.edited
    else
        case oldArt.edited of
            Just e ->
                let
                    newEd =
                        createEditable newArt
                in
                    if e == { newEd | revision = e.revision } then
                        -- the changes were applied
                        Nothing
                    else
                        -- The changes have not been applied
                        -- but the artifact has changed (by someone else)!
                        -- That's fine, keep the old edited data
                        -- and edited.revision will be used for a warning
                        Just e

            Nothing ->
                Nothing
