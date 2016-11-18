module Artifacts.Commands exposing (..)

import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Json.Decode.Pipeline exposing (decode, required, optional, hardcoded)
import Task
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (ArtifactId, Artifact, ArtifactsResponse)
import JsonRpc exposing (RpcError, formatJsonRpcError)

url = "http://localhost:4000/json-rpc"

-- COMMANDS

fetchAll : Cmd AppMsg
fetchAll =
  let
    request = Http.request
      { method = "PUT"
      , headers = 
        [ Http.header "Content-Type" "application/json"
        ]
      , url = url
      , body = Http.jsonBody <| getArtifactsRequestEncoded 1
      , expect = Http.expectJson getArtifactsResponseDecoder
      , timeout = Nothing
      , withCredentials = False
      }
  in
    Http.send newArtifactsMsg request

newArtifactsMsg : (Result Http.Error (ArtifactsResponse)) -> AppMsg
newArtifactsMsg result =
  case result of
    Ok response -> case response.result of
      Just newArtifacts -> 
        ArtifactsMsg (NewArtifacts newArtifacts)

      -- TODO: break this out to a function
      Nothing -> case response.error of
        Just error ->
          AppError (formatJsonRpcError error)
        Nothing ->
          AppError "json response had no result or error"
    
    Err err ->
      HttpError err


-- TODO: this needs to actually work...
save : Artifact -> Cmd AppMsg
save artifact = 
  let
    body = Http.jsonBody (memberEncoded artifact)

    request = Http.request
      { method = "PUT"
      , headers = 
        [ Http.header "Content-Type" "application/json"
        ]
      , url = url
      , body = body
      , expect = Http.expectJson memberDecoder
      , timeout = Nothing
      , withCredentials = False
      }
  in
    Http.send (\r -> ArtifactsMsg <| SaveArtifact r) request


-- ENCODER

getArtifactsRequestEncoded : Int -> Encode.Value
getArtifactsRequestEncoded id =
  let
    attrs =
      [ ( "jsonrpc", Encode.string "2.0" )
      , ( "id", Encode.int id )
      , ( "method", Encode.string "GetArtifacts" )
      --, ( "params", TODO: be able to fill in params
      ]
  in
    Encode.object attrs


memberEncoded : Artifact -> Encode.Value
memberEncoded artifact =
  let
    attrs =
      [ ( "id", Encode.int artifact.id )
      , ( "name", Encode.string artifact.name )
      , ( "level", Encode.int artifact.level)
      ]
  in
    Encode.object attrs
  

-- DECODERS

-- Generic RPC Error
errorDecoder : Decode.Decoder RpcError
errorDecoder =
  Decode.map2 RpcError
    (Decode.field "code" Decode.int)
    (Decode.field "message" Decode.string)


-- API Calls
getArtifactsResponseDecoder : Decode.Decoder ArtifactsResponse
getArtifactsResponseDecoder = 
  Decode.map2 ArtifactsResponse
    (Decode.maybe (Decode.field "result" collectionDecoder))
    (Decode.maybe (Decode.field "error" errorDecoder))
  

-- Generic Artifact

collectionDecoder : Decode.Decoder (List Artifact)
collectionDecoder =
  Decode.list memberDecoder

memberDecoder : Decode.Decoder Artifact
memberDecoder =
  Decode.map3 Artifact
    (Decode.field "id" Decode.int)
    (Decode.field "name" Decode.string)
    (Decode.field "level" Decode.int)
