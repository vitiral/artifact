module Artifacts.Commands exposing (..)

import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Json.Decode.Pipeline exposing (decode, required, optional, hardcoded)
import Messages exposing (AppMsg(..))
import Models exposing (Model)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing (
  ArtifactId, Artifact, Loc, ArtifactsResponse, defaultConfig, 
  Name, initName)
import JsonRpc exposing (RpcError, formatJsonRpcError)

isErr : Result e a -> Bool
isErr x =
  case x of
    Ok _ -> False
    Err _ -> True

isOk : Result e a -> Bool
isOk x =
  not (isErr x)

resultAsValue : Result String String -> String
resultAsValue x =
  case x of
    Ok r -> r
    Err e -> e

--url = "http://localhost:4000/json-rpc"
endpoint : String
endpoint = "/json-rpc"
-- COMMANDS

fetchAll : Model -> Cmd AppMsg
fetchAll model =
  let
    request = Http.request
      { method = "PUT"
      , headers = 
        [ Http.header "Content-Type" "application/json"
        ]
      , url = model.addr ++ endpoint
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
save : Model -> Artifact -> Cmd AppMsg
save model artifact = 
  let
    body = Http.jsonBody (memberEncoded artifact)

    request = Http.request
      { method = "PUT"
      , headers = 
        [ Http.header "Content-Type" "application/json"
        ]
      , url = model.addr ++ endpoint
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
    partof = List.map (\p -> p.raw) artifact.partof

    attrs =
      [ ( "id", Encode.int artifact.id )
      , ( "name", Encode.string artifact.name.raw )
      , ( "path", Encode.string artifact.path )
      , ( "text", Encode.string artifact.text )
      , ( "partof", Encode.list (List.map Encode.string partof) )
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
  decode Artifact
    |> required "id" Decode.int
    |> required "name" nameDecoder
    |> required "path" Decode.string
    |> required "text" Decode.string
    |> required "partof" (Decode.list nameDecoder)
    |> required "parts" (Decode.list nameDecoder)
    |> required "loc" (Decode.nullable locDecoder)
    |> required "completed" Decode.float
    |> required "tested" Decode.float
    |> hardcoded defaultConfig

nameDecoder : Decode.Decoder Name
nameDecoder = Decode.andThen nameDecoderValue Decode.string

nameDecoderValue : String -> Decode.Decoder Name
nameDecoderValue name =
  case initName name of
    Ok name -> 
      decode Name
        |> hardcoded name.raw
        |> hardcoded name.value
    Err err ->
      Decode.fail err

locDecoder : Decode.Decoder Loc
locDecoder =
  decode Loc
    |> required "path" Decode.string
    |> required "row" Decode.int
    |> required "col" Decode.int
