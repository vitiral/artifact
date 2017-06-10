module Update exposing (..)

import Messages exposing (AppMsg(..), formatHttpError)
import Models exposing (Model, log)
import Artifacts.Update


update : AppMsg -> Model -> (Model, Cmd AppMsg)
update msg model =
  case msg of
    ArtifactsMsg subMsg -> 
      Artifacts.Update.update subMsg model

    RouteChange route ->
      ( { model | route = route } , Cmd.none )

    -- TODO: these should do some kind of command to clear the
    -- error later
    HttpError err ->
      ( log model <| formatHttpError err, Cmd.none )

    AppError err ->
      ( log model <| "AppError: " ++ err, Cmd.none )
