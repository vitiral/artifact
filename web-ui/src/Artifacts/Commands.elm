module Artifacts.Commands exposing (..)

import Http
import Dict
import Json.Decode as Decode
import Json.Encode as Encode
import Json.Decode.Pipeline exposing (decode, required, optional, hardcoded)
import Messages exposing (AppMsg(..))
import Models exposing (Model)
import Artifacts.Messages exposing (..)
import Artifacts.Models exposing 
  ( NameKey, Artifact, Artifacts, Loc, ArtifactsResponse
  , Name, initName, artifactsFromList)
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
      , body = Http.jsonBody <| getArtifactsRequestEncoded 1 -- TODO: use better RPC id
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


saveAll : Model -> Artifacts -> Cmd AppMsg
saveAll model artifacts = 
  let
    artList = Dict.values artifacts

    -- TODO: use better RPC id
    body = Http.jsonBody (updateArtifactsRequestEncoded 1 artList)

    request = Http.request
      { method = "PUT"
      , headers = 
        [ Http.header "Content-Type" "application/json"
        ]
      , url = model.addr ++ endpoint
      , body = body
      , expect = Http.expectJson getArtifactsResponseDecoder
      , timeout = Nothing
      , withCredentials = False
      }
  in
    Http.send newArtifactsMsg request

-- REQUESTS

getArtifactsRequestEncoded : Int -> Encode.Value
getArtifactsRequestEncoded rpc_id =
  let
    attrs =
      [ ( "jsonrpc", Encode.string "2.0" )
      , ( "id", Encode.int rpc_id )
      , ( "method", Encode.string "GetArtifacts" )
      ]
  in
    Encode.object attrs


updateArtifactsRequestEncoded : Int -> List Artifact -> Encode.Value
updateArtifactsRequestEncoded rpc_id artifacts =
  let
    params = 
      Encode.object 
        [ ( "artifacts", artifactsEncoded artifacts)
        ]

    attrs = 
      [ ( "jsonrpc", Encode.string "2.0" )
      , ( "id", Encode.int rpc_id )
      , ( "method", Encode.string "GetArtifacts" )
      , ( "params", params )
      ]
  in
    Encode.object attrs


-- ENCODER

artifactsEncoded : List Artifact -> Encode.Value
artifactsEncoded artifacts =
  Encode.list (List.map artifactEncoded artifacts)


artifactEncoded : Artifact -> Encode.Value
artifactEncoded artifact =
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

-- WARNING: just returns nothing if json is invalid
-- must be used with trusted input only
artifactsFromStrUnsafe : String -> Artifacts
artifactsFromStrUnsafe json =
  let
    artifacts = case Decode.decodeString artifactsDecoder json of
      Ok a -> a
      Err _ -> [] 
  in
    artifactsFromList artifacts


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
    (Decode.maybe (Decode.field "result" artifactsDecoder))
    (Decode.maybe (Decode.field "error" errorDecoder))
  

-- Generic Artifact

artifactsDecoder : Decode.Decoder (List Artifact)
artifactsDecoder =
  Decode.list artifactDecoder

artifactDecoder : Decode.Decoder Artifact
artifactDecoder =
  decode Artifact
    |> required "id" Decode.int
    |> required "name" nameDecoder
    |> required "path" Decode.string
    |> required "text" Decode.string
    |> required "partof" (Decode.list nameDecoder)
    |> required "parts" (Decode.list nameDecoder)
    |> required "code" (Decode.nullable locDecoder)
    |> required "done" (Decode.nullable Decode.string)
    |> required "completed" Decode.float
    |> required "tested" Decode.float
    |> hardcoded Nothing

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
    |> required "line" Decode.int



