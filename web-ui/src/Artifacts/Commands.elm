module Artifacts.Commands exposing (..)

import Dict
import Http
import Json.Decode as Decode
import Json.Decode.Extra as Extra
import Json.Encode as Encode
import Json.Decode.Pipeline exposing (decode, required, optional, hardcoded)
import Messages exposing (AppMsg(..))
import Models exposing (Model)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (..)
import JsonRpc exposing (RpcError, formatJsonRpcError)


-- CONSTANTS


{-| example url = "<http://localhost:5373/json-rpc">
-}
endpoint : String
endpoint =
    "/json-rpc"



-- COMMANDS


{-| fetch artifacts
-}
fetchAll : Model -> Cmd AppMsg
fetchAll model =
    let
        body =
            Http.jsonBody <| getArtifactsRequestEncoded model.jsonId

        request =
            createJsonRequest model body projectResponseDecoder
    in
        Http.send gotProjectMsg request


updateArtifacts : Model -> Dict.Dict ArtifactId EditableArtifact -> Cmd AppMsg
updateArtifacts model edited =
    let
        body =
            Http.jsonBody <| updateArtifactsRequestEncoded model.jsonId edited

        request =
            createJsonRequest model body projectResponseDecoder
    in
        Http.send gotProjectMsg request


createArtifacts : Model -> List EditableArtifact -> Cmd AppMsg
createArtifacts model edited =
    let
        body =
            Http.jsonBody <| createArtifactsRequestEncoded model.jsonId edited

        request =
            createJsonRequest model body projectResponseDecoder
    in
        Http.send gotProjectMsg request


deleteArtifacts : Model -> List Artifact -> Cmd AppMsg
deleteArtifacts model artifacts =
    let
        body =
            Http.jsonBody <| deleteArtifactsRequestEncoded model.jsonId artifacts

        request =
            createJsonRequest model body projectResponseDecoder
    in
        Http.send gotProjectMsg request



-- Helpers


createJsonRequest : Model -> Http.Body -> Decode.Decoder d -> Http.Request d
createJsonRequest model body decoder =
    Http.request
        { method = "PUT"
        , headers =
            [ Http.header "Content-Type" "application/json"
            ]
        , url = endpoint
        , body = body
        , expect = Http.expectJson decoder
        , timeout = Nothing
        , withCredentials = False
        }


gotProjectMsg : Result Http.Error ProjectResponse -> AppMsg
gotProjectMsg result =
    case result of
        Ok response ->
            case response.error of
                Just error ->
                    AppError (formatJsonRpcError error)

                Nothing ->
                    case response.result of
                        Just gotProject ->
                            ArtifactsMsg (ReceivedProject gotProject)

                        Nothing ->
                            AppError "json response had no result or error"

        Err err ->
            HttpError err



-- REQUESTS


getArtifactsRequestEncoded : Int -> Encode.Value
getArtifactsRequestEncoded rpc_id =
    let
        attrs =
            [ ( "jsonrpc", Encode.string "2.0" )
            , ( "id", Encode.int rpc_id )
            , ( "method", Encode.string "ReadArtifacts" )
            ]
    in
        Encode.object attrs


updateArtifactsRequestEncoded : Int -> Dict.Dict ArtifactId EditableArtifact -> Encode.Value
updateArtifactsRequestEncoded rpc_id edited =
    let
        params =
            Encode.object
                [ ( "artifacts", artifactsEncoded <| Dict.toList edited )
                ]

        attrs =
            [ ( "jsonrpc", Encode.string "2.0" )
            , ( "id", Encode.int rpc_id )
            , ( "method", Encode.string "UpdateArtifacts" )
            , ( "params", params )
            ]
    in
        Encode.object attrs


createArtifactsRequestEncoded : Int -> List EditableArtifact -> Encode.Value
createArtifactsRequestEncoded rpc_id edited =
    let
        -- when creating artifacts, they always have id=0
        withIds =
            List.map (\a -> ( 0, a )) edited

        params =
            Encode.object
                [ ( "artifacts", artifactsEncoded withIds )
                ]

        attrs =
            [ ( "jsonrpc", Encode.string "2.0" )
            , ( "id", Encode.int rpc_id )
            , ( "method", Encode.string "CreateArtifacts" )
            , ( "params", params )
            ]
    in
        Encode.object attrs


deleteArtifactsRequestEncoded : Int -> List Artifact -> Encode.Value
deleteArtifactsRequestEncoded rpc_id artifacts =
    let
        -- when creating artifacts, they always have id=0
        ids =
            List.map (\a -> a.id) artifacts

        params =
            Encode.object
                [ ( "ids", Encode.list <| List.map Encode.int ids )
                ]

        attrs =
            [ ( "jsonrpc", Encode.string "2.0" )
            , ( "id", Encode.int rpc_id )
            , ( "method", Encode.string "DeleteArtifacts" )
            , ( "params", params )
            ]
    in
        Encode.object attrs



-- ENCODER


artifactsEncoded : List ( ArtifactId, EditableArtifact ) -> Encode.Value
artifactsEncoded edited =
    Encode.list <| List.map artifactEncoded edited


artifactEncoded : ( ArtifactId, EditableArtifact ) -> Encode.Value
artifactEncoded ( id, edited ) =
    let
        partof =
            List.map (\p -> p.raw) edited.partof

        done =
            if edited.done == "" then
                Encode.null
            else
                Encode.string edited.done

        attrs =
            [ ( "id", Encode.int id )
            , ( "revision", Encode.int edited.revision )
            , ( "name", Encode.string edited.name )
            , ( "def", Encode.string edited.def )
            , ( "text", Encode.string edited.text )
            , ( "partof", Encode.list <| List.map Encode.string partof )
            , ( "done", done )
            ]
    in
        Encode.object attrs



-- DECODERS


{-| WARNING: just returns nothing if json is invalid
must be used with trusted input only
-}
artifactsFromStrUnsafe : String -> Artifacts
artifactsFromStrUnsafe json =
    let
        artifacts =
            case Decode.decodeString artifactsDecoder json of
                Ok a ->
                    a

                Err _ ->
                    []
    in
        artifactsFromList artifacts


{-| Generic RPC Error
-}
errorDecoder : Decode.Decoder RpcError
errorDecoder =
    Decode.map2 RpcError
        (Decode.field "code" Decode.int)
        (Decode.field "message" Decode.string)



-- API Calls


projectResponseDecoder : Decode.Decoder ProjectResponse
projectResponseDecoder =
    Decode.map2 ProjectResponse
        (Decode.maybe (Decode.field "result" projectDecoder))
        (Decode.maybe (Decode.field "error" errorDecoder))



-- DECODERS


projectDecoder : Decode.Decoder ProjectData
projectDecoder =
    decode ProjectData
        |> required "artifacts" artifactsDecoder
        |> required "files" (Extra.set <| Decode.string)
        |> required "checked" Decode.string
        |> required "uuid" Decode.string


artifactsDecoder : Decode.Decoder (List Artifact)
artifactsDecoder =
    Decode.list artifactDecoder


artifactDecoder : Decode.Decoder Artifact
artifactDecoder =
    decode Artifact
        |> required "id" Decode.int
        |> required "revision" Decode.int
        |> required "name" nameDecoder
        |> required "def" Decode.string
        |> required "text" Decode.string
        |> required "subnames" (Decode.list Decode.string)
        |> required "partof" (Decode.list nameDecoder)
        |> required "parts" (Decode.list nameDecoder)
        |> required "code" (Decode.nullable fullLocsDecoder)
        |> required "done" (Decode.nullable Decode.string)
        |> required "completed" Decode.float
        |> required "tested" Decode.float
        |> hardcoded Nothing


nameDecoder : Decode.Decoder Name
nameDecoder =
    Decode.andThen nameDecoderValue Decode.string


nameDecoderValue : String -> Decode.Decoder Name
nameDecoderValue name =
    case initName name of
        Ok name ->
            decode Name
                |> hardcoded name.raw
                |> hardcoded name.value
                |> hardcoded name.ty

        Err err ->
            Decode.fail err


fullLocsDecoder : Decode.Decoder FullLocs
fullLocsDecoder =
    decode FullLocs
        |> required "root" (Decode.nullable locDecoder)
        |> required "error" (Decode.dict locDecoder)


locDecoder : Decode.Decoder Loc
locDecoder =
    decode Loc
        |> required "path" Decode.string
        |> required "line" Decode.int
