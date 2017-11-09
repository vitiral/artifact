module Update exposing (..)

import Dict
import Navigation
import Log

import Utils
import Artifacts.Update
import Artifacts.Models exposing (ArtifactId, Artifact)
import Messages exposing (AppMsg(..), formatHttpError, helpUrl, helpRepr, checkUrl)
import Models exposing (Model, LogMsg(..))


update : AppMsg -> Model -> ( Model, Cmd AppMsg )
update msg model =
    case msg of
        ArtifactsMsg subMsg ->
            Artifacts.Update.update subMsg model

        AckLogMsg index ->
            let
                ( _, logs ) =
                    Utils.popIndexUnsafe index model.logs
            in
                ( { model | logs = logs }, Cmd.none )

        RouteChange route ->
            ( { model | route = route }, Cmd.none )

        HttpError err ->
            ( Log.log model <| LogErr <| formatHttpError err, Cmd.none )

        AppError err ->
            ( Log.log model <| LogErr <| "AppError: " ++ err, Cmd.none )

        ShowHelp page ->
            let
                url =
                    "#" ++ helpUrl ++ "/" ++ (helpRepr page)
            in
                ( model, Navigation.newUrl url )

        ShowCheck ->
            ( model, Navigation.newUrl <| "#" ++ checkUrl )

        RenderArtifacts a ->
            let
                _ = Debug.log "INVALID request to render artifacts" a
            in
                (model, Cmd.none)

        ArtifactsRendered rendered ->
            let
                _ = Debug.log "SUCCESS artifacts rendered:" rendered

                renderedDict = Dict.fromList rendered

                -- update artifacts with the new rendered text
                updateRendered : ArtifactId -> Artifact -> Artifact
                updateRendered id artifact =
                    case Dict.get id renderedDict of
                        Nothing ->
                            artifact
                        Just r ->
                            { artifact | renderedText = r }

                artifacts = Dict.map updateRendered model.artifacts
            in
                ({ model | artifacts = artifacts }, Cmd.none)

        Noop ->
            ( model, Cmd.none )
