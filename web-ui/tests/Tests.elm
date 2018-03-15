module Tests exposing (..)

import Test exposing (..)
import Expect
import Fuzz exposing (list, int, tuple, string)
import String
import Dict
import Json.Decode as Decode
import Json.Encode as Encode
import Artifacts.Models
    exposing
        ( Artifact
        , initName
        , initNameUnsafe
        , Type(..)
        , autoPartof
        , getEditable
        )
import Artifacts.Commands
    exposing
        ( artifactEncoded
        , artifactDecoder
        , artifactsFromStrUnsafe
        )
import Utils


-- artifact : Artifact
-- artifact =
--     { id = 10
--     , revision = 0
--     , name = { value = "REQ-NAME", raw = "req-name", ty = Req }
--     , def = "path"
--     , text = "text"
--     , subnames = []
--     , partof = [ { value = "REQ-PARTOF-1", raw = "req-partof-1", ty = Req } ]
--     , parts = [ { value = "REQ-PART-1", raw = "req-part-1", ty = Req } ]
--     , code = Just { root = Just { path = "path", line = 10 }, sublocs = Dict.empty }
--     , done = Nothing
--     , completed = 0.0
--     , tested = 0.0
--     , edited = Nothing
--     }
-- 
-- 
-- expectedEncoded =
--     "{\"id\":10,\"revision\":0,\"name\":\"req-name\",\"def\":\"path\",\"text\":\"text\",\"partof\":[\"req-partof-1\"],\"done\":null}"
-- 
-- 
-- artifactsJson =
--     """
--   [
--     { "id": 10
--     , "revision": 0
--     , "name":"req-name"
--     , "def":"path"
--     , "text": "text"
--     , "subnames": []
--     , "partof": ["req-partof-1"]
--     , "parts": ["req-part-1"]
--     , "code":
--         { "root": { "path": "path", "line": 10 }
--         , "sublocs": {}
--         }
--     , "done": null
--     , "completed": 0.0
--     , "tested": 0.0
--     }
--   ]
--   """


nameValid : String -> Bool
nameValid name =
    case initName name of
        Ok _ ->
            True

        Err _ ->
            False


namesValid : List String -> List Bool
namesValid names =
    List.map nameValid names


testJson =
    describe "Json Tests"
        [ describe "json: serialzation -> deserialization of models"
            [ test "Addition" <|
                \() ->
                    Expect.equal (3 + 7) 10
            -- , test "encode artifact" <|
            --     \() ->
            --         let
            --             encoded =
            --                 artifactEncoded ( artifact.id, getEditable artifact )

            --             result =
            --                 Encode.encode 0 encoded
            --         in
            --             Expect.equal result expectedEncoded
            -- , test "decode artifact" <|
            --     \() ->
            --         let
            --             expected =
            --                 Dict.singleton 10 artifact
            --         in
            --             Expect.equal (artifactsFromStrUnsafe artifactsJson) expected
            ]
        , describe "name: test name validation"
            [ test "valid names 1" <|
                \() ->
                    Expect.equalLists
                        (namesValid [ "REQ-foo", "REQ-foo-2", "REQ-foo2" ])
                        ([ True, True, True ])
            , test "valid names 2" <|
                \() ->
                    Expect.equalLists
                        (namesValid [ "REQ-foo-bar-2_3", "SPC-foo", "TST-foo" ])
                        ([ True, True, True ])
            , test "invalid names 1" <|
                \() ->
                    Expect.equalLists
                        (namesValid [ "REQ-foo*", "REQ-foo\n", "REQ-foo-" ])
                        ([ False, False, False ])
            ]
        ]


testUtils =
    let
        testList =
            [ "a", "b", "c" ]
    in
        describe "Test Utils"
            [ test "assertOr" <|
                \() ->
                    Expect.equal (Utils.assertOr True 3 "msg") 3
            , test "enumerate" <|
                \() ->
                    Expect.equal (Utils.enumerate testList) [ ( 0, "a" ), ( 1, "b" ), ( 2, "c" ) ]
            , test "setIndexUnsafe-1" <|
                \() ->
                    Expect.equal (Utils.setIndexUnsafe 0 "z" testList) [ "z", "b", "c" ]
            , test "setIndexUnsafe-2" <|
                \() ->
                    Expect.equal (Utils.setIndexUnsafe 2 "g" testList) [ "a", "b", "g" ]
            , test "popIndexUnsafe-start" <|
                \() ->
                    Expect.equal (Utils.popIndexUnsafe 0 testList) ( "a", [ "b", "c" ] )
            , test "popIndexUnsafe-mid" <|
                \() ->
                    Expect.equal (Utils.popIndexUnsafe 1 testList) ( "b", [ "a", "c" ] )
            , test "popIndexUnsafe-end" <|
                \() ->
                    Expect.equal (Utils.popIndexUnsafe 2 testList) ( "c", [ "a", "b" ] )
            ]


testPartof =
    let
        req_name =
            initNameUnsafe "REQ-name"

        req_name_foo =
            initNameUnsafe "REQ-name-foo"

        req_foo =
            initNameUnsafe "REQ-foo"

        req_foo_bar =
            initNameUnsafe "REQ-foo-bar"

        req_foo_bar_baz =
            initNameUnsafe "REQ-foo-bar-baz"

        spc_name =
            initNameUnsafe "SPC-name"
    in
        describe "Test Partof"
            [ test "req-name-foo automatically has partof req-name" <|
                \() ->
                    Expect.equal True <| autoPartof req_name_foo req_name
            , test "not vice versa" <|
                \() ->
                    Expect.equal False <| autoPartof req_name req_name_foo
            , test "req-foo does NOT automatically have partof req-name" <|
                \() ->
                    Expect.equal False <| autoPartof req_foo req_name
            , test "spc-name automatically has partof req-name" <|
                \() ->
                    Expect.equal True <| autoPartof spc_name req_name
            , test "not vice versa" <|
                \() ->
                    Expect.equal False <| autoPartof req_name spc_name
            , test "auto goes back one layer" <|
                \() ->
                    Expect.equal True <| autoPartof req_foo_bar req_foo
            , test "auto does NOT go back two layers" <|
                \() ->
                    Expect.equal False <| autoPartof req_foo_bar_baz req_foo
            ]


all : Test
all =
    describe "Artifact Test Suite"
        [ testJson
        , testUtils
        , testPartof
        ]
