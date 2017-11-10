module Update exposing (..)

import Navigation
import Log
import Utils
import Artifacts.Update
import Artifacts.TextLinks exposing (replaceArtifactLinks)
import Messages exposing (AppMsg(..), formatHttpError, helpUrl, helpRepr, checkUrl)
import Models exposing (Model, LogMsg(..), getViewingText)
import Ports


update : AppMsg -> Model -> ( Model, Cmd AppMsg )
update msg model =
    case msg of
        ArtifactsMsg subMsg ->
            let
                ( new_model, new_cmd ) =
                    Artifacts.Update.update subMsg model
            in
                requestRerender new_model [ new_cmd ]

        AckLogMsg index ->
            let
                ( _, logs ) =
                    Utils.popIndexUnsafe index model.logs
            in
                ( { model | logs = logs }, Cmd.none )

        RouteChange route ->
            let
                new_model =
                    { model | route = route }
            in
                requestRerender new_model []

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

        RenderText text ->
            let
                _ =
                    Debug.log "INVALID request to render artifacts" text
            in
                ( model, Cmd.none )

        TextRendered rendered ->
            ( { model | renderedText = Just rendered }, Cmd.none )

        Noop ->
            ( model, Cmd.none )


requestRerender : Model -> List (Cmd AppMsg) -> ( Model, Cmd AppMsg )
requestRerender model cmds =
    let
        -- Make a call to get the text rendered AND invalidate
        -- the existing rendered text
        final_model =
            { model | renderedText = Nothing }

        renderCmds =
            case getViewingText model of
                Just text ->
                    [ Ports.renderText <| replaceArtifactLinks model text ]

                Nothing ->
                    []

        final_cmds =
            List.append renderCmds cmds
    in
        ( final_model, Cmd.batch final_cmds )
