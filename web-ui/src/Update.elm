module Update exposing (..)

import Messages exposing (AppMsg(..), formatHttpError)
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

        -- TODO: these should do some kind of command to clear the
        -- error later
        HttpError err ->
            ( Log.log model <| LogErr <| formatHttpError err, Cmd.none )

        AppError err ->
            ( Log.log model <| LogErr <| "AppError: " ++ err, Cmd.none )

        Noop ->
            ( model, Cmd.none )
