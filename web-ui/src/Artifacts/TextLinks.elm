module Artifacts.TextLinks exposing (..)

import Regex
import Models exposing (Model, getArtifact, memberArtifact, getCreateArtifact)
import Artifacts.Models exposing (..)
import Utils
import Artifacts.PartGraph exposing (nameNodeSettings)


{-| regex to search for and replace [[ART-name]]
-}
artifactLinkRegex : Regex.Regex
artifactLinkRegex =
    Regex.caseInsensitive <| Regex.regex <| "\\[\\[(\\w+:)?(" ++ artifactValidRaw ++ ")\\]\\]"


{-| replace [[ART-name]] with [ART-name](link)
-}
replaceArtifactLinks : Model -> String -> String
replaceArtifactLinks model text =
    let
        replace : Regex.Match -> String
        replace match =
            let
                name = Utils.getIndexUnsafe 1 match.submatches
                    |> Utils.unwrap "match artifact name"
            in
                case Utils.getIndexUnsafe 0 match.submatches of
                    Just "dot:" ->
                        nameNodeSettings model name

                    Just unknown ->
                        "[[ARTIFACT ERROR - format type \"" ++ unknown ++ "\" is unknown]]"

                    Nothing ->
                        if memberArtifact name model then
                            "[" ++ name ++ "](" ++ (fullArtifactUrl model name) ++ ")"
                        else
                            ("<strike style=\"color:red\", "
                                ++ "title=\"artifact name not found\">[["
                                ++ name
                                ++ "]]</strike>"
                            )

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
