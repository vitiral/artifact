module Messages exposing (..)

import Http
import Artifacts.Messages


-- CONSTANTS


createUrl : String
createUrl =
    "create"



-- TYPES


type Route
    = ArtifactsRoute
    | ArtifactNameRoute String
    | ArtifactCreateRoute
    | NotFoundRoute


type AppMsg
    = ArtifactsMsg Artifacts.Messages.Msg
    | RouteChange Route
    | HttpError Http.Error
    | AppError String
    | Noop


formatHttpError : Http.Error -> String
formatHttpError error =
    case error of
        Http.BadPayload desc resp ->
            "HTTP Error BadPayload: " ++ desc

        Http.BadUrl url ->
            "HTTP Error BadUrl: " ++ url

        Http.Timeout ->
            "HTTP Error Timeout"

        Http.NetworkError ->
            "HTTP Error NetworkError"

        Http.BadStatus response ->
            "HTTP Error BadStatus: " ++ response.body
