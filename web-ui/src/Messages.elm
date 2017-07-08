module Messages exposing (..)

import Http
import Artifacts.Messages


-- CONSTANTS


createUrl : String
createUrl =
    "create"


editingUrl : String
editingUrl =
    "unsaved"


helpUrl : String
helpUrl =
    "help"


checkUrl : String
checkUrl =
    "check"



-- TYPES


type Route
    = ArtifactsRoute
    | ArtifactNameRoute String
    | ArtifactCreateRoute
    | ArtifactEditingRoute
    | CheckRoute
    | HelpRoute String
    | NotFoundRoute


type HelpPage
    = HelpMain
    | HelpName
    | HelpParts
    | HelpPartof
    | HelpText
    | HelpDefined
    | HelpImplemented
    | HelpDone
    | HelpEdit


helpRepr : HelpPage -> String
helpRepr page =
    case page of
        HelpMain ->
            ""

        HelpName ->
            "name"

        HelpParts ->
            "parts"

        HelpPartof ->
            "partof"

        HelpText ->
            "text"

        HelpDefined ->
            "defined"

        HelpImplemented ->
            "implemented"

        HelpDone ->
            "done"

        HelpEdit ->
            "edit"


type AppMsg
    = ArtifactsMsg Artifacts.Messages.Msg
    | AckLogMsg Int
    | RouteChange Route
    | HttpError Http.Error
    | AppError String
    | ShowHelp HelpPage
    | ShowCheck
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
