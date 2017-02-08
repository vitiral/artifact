-- #TST-web-unit

module Tests exposing (..)

import Test exposing (..)
import Expect
import Fuzz exposing (list, int, tuple, string)
import String
import Json.Decode as Decode
import Json.Encode as Encode

import Artifacts.Models exposing (Artifact, defaultConfig, initName)
import Artifacts.Commands exposing (artifactEncoded, artifactDecoder)


artifact : Artifact
artifact =
  { id = 10
  , name = { value = "REQ-NAME", raw = "req-name" }
  , path = "path"
  , text = "text"
  , partof = [ {value = "REQ-PARTOF-1", raw = "req-partof-1"} ]
  , parts = [ {value = "REQ-PART-1", raw = "req-part-1"} ]
  , loc = Just { path = "path", row = 10, col = 10 }
  , completed = 0.0
  , tested = 0.0
  , config = defaultConfig
  , edited = Nothing
  }

expectedEncoded = 
  "{\"id\":10,\"name\":\"req-name\",\"path\":\"path\",\"text\":\"text\",\"partof\":[\"req-partof-1\"]}"

artifactJson =
  """
  { "id":10
  , "name":"req-name"
  , "path":"path"
  , "text": "text"
  , "partof": ["req-partof-1"]
  , "parts": ["req-part-1"]
  , "loc": { "path": "path", "row": 10, "col": 10 }
  , "completed": 0.0
  , "tested": 0.0
  , "edited": null
  }
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
          Expect.equal (Decode.decodeString artifactDecoder artifactJson) (Ok artifact)
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
