-- #TST-web-unit

module Tests exposing (..)

import Test exposing (..)
import Expect
import Fuzz exposing (list, int, tuple, string)
import String
import Dict
import Json.Decode as Decode
import Json.Encode as Encode

import Artifacts.Models exposing (Artifact, initName)
import Artifacts.Commands exposing (
  artifactEncoded, artifactDecoder, artifactsFromStrUnsafe)


artifact : Artifact
artifact =
  { id = 10
  , name = { value = "REQ-NAME", raw = "req-name" }
  , path = "path"
  , text = "text"
  , partof = [ {value = "REQ-PARTOF-1", raw = "req-partof-1"} ]
  , parts = [ {value = "REQ-PART-1", raw = "req-part-1"} ]
  , code = Just { path = "path", line = 10 }
  , done = Nothing
  , completed = 0.0
  , tested = 0.0
  , edited = Nothing
  }

expectedEncoded = 
  "{\"id\":10,\"name\":\"req-name\",\"path\":\"path\",\"text\":\"text\",\"partof\":[\"req-partof-1\"]}"

artifactsJson =
  """
  [
    { "id":10
    , "name":"req-name"
    , "path":"path"
    , "text": "text"
    , "partof": ["req-partof-1"]
    , "parts": ["req-part-1"]
    , "code": { "path": "path", "line": 10 }
    , "done": null
    , "completed": 0.0
    , "tested": 0.0
    }
  ]
  """

nameValid : String -> Bool
nameValid name =
  case initName name of
    Ok _ -> True
    Err _ -> False

namesValid : List String -> List Bool
namesValid names =
  List.map nameValid names

all : Test
all =
  describe "RST test suite"
    [ describe "json: serialzation -> deserialization of models"
      [ test "Addition" <|
        \() ->
          Expect.equal (3 + 7) 10
      , test "encode artifact" <|
        \() ->
          Expect.equal (Encode.encode 0 (artifactEncoded artifact)) expectedEncoded
      , test "decode artifact" <|
        \() ->
          let
            expected = Dict.singleton "REQ-NAME" artifact
          in
            Expect.equal (artifactsFromStrUnsafe artifactsJson) expected
      ]
    , describe "name: test name validation"
      [ test "valid names 1" <|
        \() ->
          Expect.equalLists
            ( namesValid ["REQ-foo", "REQ-foo-2", "REQ-foo2"] )
            ( [True, True, True] )
      , test "valid names 2" <|
        \() ->
          Expect.equalLists
            ( namesValid ["REQ-foo-bar-2_3", "SPC-foo", "RSK-foo", "TST-foo"] )
            ( [True, True, True, True] )
      , test "invalid names 1" <|
        \() ->
          Expect.equalLists
            ( namesValid ["REQ-foo*", "REQ-foo\n", "REQ-foo-"] )
            ( [False, False, False] )
      ]
    ]

