module Artifacts.View exposing (..)

{-
   Generic view methods that can be used in multiple places (List, Edit, etc)
-}

import String
import Html exposing (..)
import Html.Attributes exposing (value, class, href, title, id, selected)
import Html.Events exposing (onClick, onInput)
import Messages exposing (AppMsg(..), Route(..))
import Models exposing (Model, getArtifact, memberArtifact)
import Artifacts.Models
    exposing
        ( Artifact
        , Name
        , Type(..)
        , EditableArtifact
        , artifactNameUrl
        , indexName
        , indexNameUnchecked
        , initNameUnsafe
        , initName
        , getType
        , autoPartof
        , validPartof
        )
import Artifacts.Messages exposing (Msg(..))


completion : Artifact -> Html AppMsg
completion artifact =
    div [ class "clearfix py1" ]
        [ div [ class "col col-6" ]
            [ span [ class "bold" ] [ text "Implemented: " ]
            , completedPerc artifact
            ]
        , div [ class "col col-6" ]
            [ span [ class "bold" ] [ text "Tested: " ]
            , testedPerc artifact
            ]
        ]


completedPerc : Artifact -> Html msg
completedPerc artifact =
    let
        score =
            completedScore artifact

        color =
            if score >= 3 then
                "olive"
            else if score >= 2 then
                "blue"
            else if score >= 1 then
                "orange"
            else
                "red"
    in
        span [ class ("bold " ++ color) ]
            [ text <| (String.left 3 (toString (artifact.completed * 100))) ++ "%" ]


testedPerc : Artifact -> Html msg
testedPerc artifact =
    let
        score =
            testedScore artifact

        color =
            if score >= 2 then
                "olive"
            else if score >= 1 then
                "orange"
            else
                "red"
    in
        span [ class ("bold " ++ color) ]
            [ text <| (String.left 3 (toString (artifact.tested * 100))) ++ "%" ]


implemented : Model -> Artifact -> Html m
implemented model artifact =
    div []
        (case ( artifact.code, artifact.done ) of
            ( Just loc, Nothing ) ->
                [ span [ class "bold" ] [ text "Implemented at: " ]
                , implementedBasic model artifact
                ]

            ( Nothing, Just done ) ->
                [ span [ class "bold" ] [ text "Defined as done: " ]
                , implementedBasic model artifact
                ]

            ( Nothing, Nothing ) ->
                [ span [ class "bold" ] [ text "Implementation: " ]
                , implementedBasic model artifact
                ]

            ( Just _, Just _ ) ->
                [ implementedBasic model artifact ]
        )


{-| show implemented with some settings
-- TODO: enable editing
-}
implementedBasic : Model -> Artifact -> Html m
implementedBasic model artifact =
    let
        ( s, t ) =
            case ( artifact.code, artifact.done ) of
                ( Just loc, Nothing ) ->
                    ( [], loc.path ++ "[" ++ (toString loc.line) ++ "]" )

                ( Nothing, Just done ) ->
                    ( [], done )

                ( Nothing, Nothing ) ->
                    ( [ class "italic gray" ], "not directly implemented" )

                ( Just _, Just _ ) ->
                    -- TODO: send error message
                    ( [ class "bold red" ], "ERROR: code+done both set" )
    in
        span (s ++ [ getId "implemented" artifact Nothing ]) [ text t ]



-- for the full Edit view


parts : Model -> Artifact -> Html AppMsg
parts model artifact =
    ul
        [ getId "parts" artifact Nothing ]
        (List.map
            (\p ->
                li
                    [ class "underline" ]
                    [ seeArtifactName model p artifact "parts" False ]
            )
            artifact.parts
        )


{-| TODO: don't wrap text
-}
textPiece : Model -> Artifact -> Html AppMsg
textPiece model artifact =
    let
        ro =
            model.settings.readonly

        text_part =
            String.left 200 artifact.text

        t =
            if (String.length artifact.text) > 200 then
                text_part ++ " ..."
            else
                text_part
    in
        text text_part


testedScore : Artifact -> Int
testedScore artifact =
    if artifact.tested >= 1.0 then
        2
    else if artifact.tested >= 0.5 then
        1
    else
        0


completedScore : Artifact -> Int
completedScore artifact =
    if artifact.completed >= 1.0 then
        3
    else if artifact.completed >= 0.7 then
        2
    else if artifact.completed >= 0.4 then
        1
    else
        0


artifactColor : Artifact -> String
artifactColor artifact =
    let
        score =
            (testedScore artifact) + (completedScore artifact)
    in
        if score >= 5 then
            "olive"
        else if score >= 3 then
            "blue"
        else if score >= 1 then
            "orange"
        else
            "red"


{-| colors: olive, blue, orange, red
-}
seeArtifact : Model -> Artifact -> Html AppMsg
seeArtifact model artifact =
    a
        [ class ("btn bold " ++ (artifactColor artifact))
        , id artifact.name.value
        , onClick (ArtifactsMsg <| ShowArtifact <| artifact.name.value)
        , href (artifactNameUrl artifact.name.value)
        ]
        [ text (artifact.name.raw ++ "  ") ]


{-| TODO: do color and other special stuff for non-existent names
-}
seeArtifactName : Model -> Name -> Artifact -> String -> Bool -> Html AppMsg
seeArtifactName model name parent attr ed =
    let
        url =
            artifactNameUrl name.value

        color =
            case getArtifact name.value model of
                Just a ->
                    artifactColor a

                Nothing ->
                    "purple"

        prefix =
            case ed of
                True ->
                    "ed_"

                False ->
                    "rd_"
    in
        if memberArtifact name.value model then
            a
                [ class ("btn bold " ++ color)
                , id <| prefix ++ attr ++ "_" ++ parent.name.value ++ "_" ++ name.value
                , href url
                , onClick <| ArtifactsMsg <| ShowArtifact name.value
                , title <|
                    if memberArtifact name.value model then
                        "goto artifact"
                    else
                        "artifact not found"
                ]
                [ text name.raw ]
        else
            span [ class ("btn " ++ color) ] [ text name.raw ]



------------------------
-- Helpers


{-| get the id html attribute
-}
getId : String -> Artifact -> Maybe EditableArtifact -> Attribute m
getId attr artifact edited =
    if edited == Nothing then
        id ("rd_" ++ attr ++ "_" ++ artifact.name.value)
        -- read
    else
        id ("ed_" ++ attr ++ "_" ++ artifact.name.value)
