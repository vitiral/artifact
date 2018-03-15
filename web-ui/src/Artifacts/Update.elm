module Artifacts.Update exposing (..)

import Dict
import Navigation
import Models exposing (..)
import Messages
    exposing
        ( createUrl
        , editingUrl
        , AppMsg(AppError)
        , Route(..)
        )
import Utils exposing (assertOr)
import Log
import Artifacts.Messages exposing (Msg(..))
import Artifacts.Models exposing (..)
import Artifacts.Commands exposing (updateArtifacts, createArtifacts, deleteArtifacts)


mismatchedUuidMsg : String
mismatchedUuidMsg =
    ("Mismatched server uuid: the server has been restarted."
        ++ " Copy all unsaved data and reload this tab."
        ++ " ALL UNSAVED DATA WILL BE LOST!"
    )


update : Msg -> Model -> ( Model, Cmd AppMsg )
update msg model =
    case msg of
        ReceivedProject project ->
            let
                ( new_model, cmds ) =
                    handleReceived model project.artifacts

                -- add final attributes
                final_model =
                    { new_model
                        | files = project.paths.artifact_paths
                        , checked = project.checked
                        , uuid = "FIXME: no real uuid"
                    }
            in
                if model.uuid == "" || model.uuid == final_model.uuid then
                    ( final_model, Cmd.batch cmds )
                else
                    -- the uuid changed, ignore everything and log an error
                    let
                        err_model =
                            Log.log model <| LogErr mismatchedUuidMsg
                    in
                        ( err_model, Cmd.none )

        ShowArtifacts ->
            ( model, Navigation.newUrl artifactsUrl )

        ShowArtifact name ->
            ( model
            , Navigation.newUrl <|
                artifactNameUrl (indexNameUnchecked name)
            )

        ShowEditing ->
            ( model
            , Navigation.newUrl <| "#" ++ editingUrl
            )

        CreateArtifact ->
            ( model
            , Navigation.newUrl <| "#" ++ createUrl
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

        EditArtifact option ->
            case option of
                ChangeChoice artifact edited ->
                    let
                        -- update original_id so that any warnings of prior
                        -- change go away
                        e =
                            { edited | original_id = artifact.id }

                        artifacts =
                            setEdited model.artifacts artifact (Just e)
                    in
                        ( { model | artifacts = artifacts }
                        , Cmd.none
                        )

                CreateChoice edited ->
                    ( { model | create = Just edited }, Cmd.none )

        CancelEditArtifact option ->
            case option of
                ChangeChoice artifact _ ->
                    ( { model | artifacts = setEdited model.artifacts artifact Nothing }
                    , Cmd.none
                    )

                CreateChoice _ ->
                    ( { model | create = Nothing }, Cmd.none )

        SaveArtifact option ->
            case option of
                ChangeChoice artifact edited ->
                    let
                        value =
                            Dict.singleton artifact.id (getEditable artifact)
                    in
                        ( { model | jsonId = model.jsonId + 1 }
                        , updateArtifacts model value
                        )

                CreateChoice edited ->
                    let
                        model2 =
                            { model | jsonId = model.jsonId + 1 }
                    in
                        ( model2, createArtifacts model [ edited ] )

        DeleteArtifact artifact ->
            ( model, deleteArtifacts model [ artifact ] )


{-| set the edited variable on the requested artifact
-}
setEdited : Artifacts -> Artifact -> Maybe EditableArtifact -> Artifacts
setEdited artifacts art edited =
    Dict.insert art.id { art | edited = edited } artifacts


{-| we need to make sure we keep any edited data that has not been applied
-}
handleReceived : Model -> List Artifact -> ( Model, List (Cmd AppMsg) )
handleReceived model artifactList =
    let
        processed =
            List.map (processNew model) artifactList

        -- get the artifacts, removing artifacts
        -- that don't exist in the given ids
        artifacts =
            List.map (\p -> p.artifact) processed
                |> artifactsFromList

        names =
            nameIds artifacts

        routes =
            List.filterMap (\p -> p.route) processed

        clear_create =
            List.map (\p -> p.clear_create) processed
                |> List.any (\a -> a)

        logs =
            model.logs ++ (List.filterMap (\p -> p.log) processed)

        _ =
            assertOr ((List.length routes) <= 1) 0 "impossible routes"

        ( route, cmds ) =
            case List.head routes of
                Just r ->
                    ( ArtifactNameRoute r
                    , [ Navigation.newUrl <| artifactNameUrl r ]
                    )

                Nothing ->
                    ( model.route, [] )

        create =
            if clear_create then
                Nothing
            else
                model.create

        new_model =
            { model
                | artifacts = artifacts
                , names = names
                , route = route
                , create = create
                , logs = logs
            }

        -- the artifact may have been deleted in which case, log a message and
        -- switch to list
        final_model =
            case new_model.route of
                ArtifactNameRoute rawName ->
                    if Dict.member (indexNameUnchecked rawName) new_model.names then
                        new_model
                    else
                        let
                            logged =
                                "Artifact Deletion Successful: "
                                    ++ rawName
                                    |> LogOk
                                    |> Log.log new_model
                        in
                            { logged | route = ArtifactsRoute }

                _ ->
                    new_model
    in
        ( final_model, cmds )


{-| get the edited, keeping in mind that changes may have been applied
-}
handleEditedReceived : Artifact -> Artifact -> Maybe EditableArtifact
handleEditedReceived oldArt newArt =
    if oldArt.id == newArt.id then
        oldArt.edited
    else
        case oldArt.edited of
            Just e ->
                if editedEqual e <| createEditable newArt then
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


processNew :
    Model
    -> Artifact
    ->
        { log : Maybe LogMsg
        , clear_create : Bool
        , route : Maybe String
        , artifact : Artifact
        }
processNew model newArt =
    case Dict.get newArt.id model.artifacts of
        Just oldArt ->
            -- artifact exists and is being updated
            let
                edited =
                    handleEditedReceived oldArt newArt

                route =
                    case model.route of
                        ArtifactNameRoute route_name ->
                            if (indexNameUnchecked route_name) == oldArt.name.value then
                                -- Artifact we are viewing got changed. Always go to
                                -- the new name
                                Just newArt.name.value
                            else
                                Nothing

                        _ ->
                            Nothing

                log =
                    if oldArt.id == newArt.id then
                        Nothing
                    else
                        Just <| LogOk <| "Artifact Update Successful: " ++ newArt.name.raw
            in
                { clear_create = False
                , route = route
                , artifact = { newArt | edited = edited }
                , log = log
                }

        Nothing ->
            -- artifact is new
            let
                edited =
                    createEditable newArt

                ( log, clear ) =
                    case model.create of
                        Just e ->
                            if editedEqual e edited then
                                ( Just <| LogOk <| "Artifact Creation Successful: " ++ newArt.name.raw
                                , True
                                )
                            else
                                ( Nothing, False )

                        _ ->
                            ( Nothing, False )
            in
                { clear_create = clear
                , route = Nothing
                , artifact = newArt
                , log = log
                }
