module Update exposing (..)

import Navigation
import Messages exposing (AppMsg(..), formatHttpError, helpUrl, helpRepr, checkUrl)
import Models exposing (Model, LogMsg(..))
import Log
import Utils
import Artifacts.Update


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

        Noop ->
            ( model, Cmd.none )
