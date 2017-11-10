module Artifacts.TextLinks exposing (..)

import Regex
import Dict
import Models exposing (Model, getArtifact, memberArtifact, getCreateArtifact)
import Artifacts.Models exposing (..)


{-| regex to search for and replace [[ART-name]]
-}
artifactLinkRegex : Regex.Regex
artifactLinkRegex =
    Regex.caseInsensitive <| Regex.regex <| "\\[\\[(" ++ artifactValidRaw ++ ")\\]\\]"


{-| replace [[ART-name]] with [ART-name](link)
-}
replaceArtifactLinks : Model -> String -> String
replaceArtifactLinks model text =
    let
        replace : Regex.Match -> String
        replace match =
            case List.head match.submatches of
                Just m ->
                    case m of
                        Just m ->
                            if Dict.member (indexNameUnchecked m) model.names then
                                "[" ++ m ++ "](" ++ (fullArtifactUrl model m) ++ ")"
                            else
                                ("<strike style=\"color:red\", "
                                    ++ "title=\"artifact name not found\">[["
                                    ++ m
                                    ++ "]]</strike>"
                                )

                        Nothing ->
                            "INTERNAL_ERROR"

                Nothing ->
                    "INTERNAL_ERROR"
    in
        Regex.replace Regex.All artifactLinkRegex replace text


{-| get the full url to a single artifact
-}
fullArtifactUrl : Model -> String -> String
fullArtifactUrl model indexName =
    let
        addrName =
            String.toLower (indexNameUnchecked indexName)

        -- super hacky way to get the origin: might fail for files
        -- I tried location.origin... doesn't work for some reason.
        -- neither does location.host + location.pathname
        origin =
            case List.head (String.split "#" model.location.href) of
                Just o ->
                    removeSlashEnd o

                Nothing ->
                    "ERROR-origin-no-head"
    in
        origin ++ "/" ++ artifactsUrl ++ "/" ++ addrName


removeSlashEnd : String -> String
removeSlashEnd path =
    if String.endsWith "/" path then
        removeSlashEnd (String.dropRight 1 path)
    else
        path
