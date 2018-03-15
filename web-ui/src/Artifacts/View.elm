module Artifacts.View exposing (..)

{-
   Generic view methods that can be used in multiple places (List, Edit, etc)
-}

import Dict
import String
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput)
import Markdown exposing (toHtml)
import Models exposing (Model, getArtifact, memberArtifact, getCreateArtifact)
import Messages exposing (createUrl, AppMsg(..), HelpPage(..), Route(..))
import Utils
import Artifacts.Models exposing (..)
import Artifacts.Nav as Nav
import Artifacts.Messages exposing (Msg(ShowArtifact, CreateArtifact))


{-| Display a list of all artifacts that are currently being edited.
-}
viewEditing : Model -> Html AppMsg
viewEditing model =
    let
        creating : List (Html AppMsg)
        creating =
            case model.create of
                Just c ->
                    [ li []
                        [ Nav.editBtn <| EditChoice <| CreateChoice c
                        , a
                            [ class "btn bold"
                            , id <| "CREATE_" ++ c.name
                            , onClick <| ArtifactsMsg <| CreateArtifact
                            , href <| "#" ++ createUrl
                            ]
                            [ text <| "Creating " ++ c.name ]
                        ]
                    ]

                Nothing ->
                    []

        line artifact =
            case artifact.edited of
                Just e ->
                    Just
                        (li []
                            [ Nav.editBtn <| EditChoice <| ChangeChoice artifact e
                            , seeArtifact model artifact
                            ]
                        )

                Nothing ->
                    Nothing

        lines =
            Dict.values model.artifacts
                |> List.filterMap line

        editing =
            ul []
                (creating ++ lines)

        header =
            h1
                [ class "h1" ]
                [ Nav.helpBtn HelpEdit False
                , text "Artifacts you have not yet saved."
                ]
    in
        div [ id "editing_view" ]
            [ Nav.bar model <| Nav.editingBar model
            , header
            , editing
            ]


displayRenderedText : Model -> ViewOption -> Html AppMsg
displayRenderedText model option =
    let
        thisId =
            idAttr "rendered_text" option
    in
        case model.rendered of
            Just r ->
                toHtml [ thisId ] r.text

            Nothing ->
                div [ thisId ] []


displayRenderedFamily : Model -> ViewOption -> Html AppMsg
displayRenderedFamily model option =
    let
        thisId =
            idAttr "rendered_part" option
    in
        case model.rendered of
            Just r ->
                toHtml [ thisId ] r.part

            Nothing ->
                div [ thisId ] []


viewIdAttr : ViewOption -> Attribute m
viewIdAttr option =
    id <|
        case option of
            ReadChoice _ ->
                "read_view"

            EditChoice choice ->
                case choice of
                    ChangeChoice _ _ ->
                        "edit_view"

                    CreateChoice _ ->
                        "create_view"


{-| display a warning if the artifact changed from under the user
-}
revisionWarnings : Model -> ViewOption -> List (Html AppMsg)
revisionWarnings model option =
    case option of
        ReadChoice _ ->
            []

        EditChoice choice ->
            case choice of
                ChangeChoice artifact edited ->
                    if artifact.id == edited.original_id then
                        []
                    else
                        [ h1
                            [ class "h1 red"
                            , id "warn_edit_change"
                            ]
                            [ text <|
                                "!! This artifact has been changed"
                                    ++ " by another user since editing"
                                    ++ " started !!"
                            ]
                        ]

                CreateChoice _ ->
                    []


viewCompletedPerc : Artifact -> List (Html AppMsg)
viewCompletedPerc artifact =
    [ span [ class "bold" ]
        [ Nav.helpBtn HelpParts False
        , text "Completed:"
        ]
    , completedPerc artifact
    ]


viewTestedPerc : Artifact -> List (Html AppMsg)
viewTestedPerc artifact =
    [ span [ class "bold" ]
        [ Nav.helpBtn HelpParts False
        , text "Tested:"
        ]
    , testedPerc artifact
    ]


colorAttr : String -> Attribute msg
colorAttr color =
    style [ ( "color", color ) ]


oliveColor : String
oliveColor =
    "#3da03d"


blueColor : String
blueColor =
    "#0074D9"


orangeColor : String
orangeColor =
    "#FF851B"


redColor : String
redColor =
    "#FF4136"


purpleColor : String
purpleColor =
    "#B10DC9"


{-| FIXME: delete this
-}
completion : Artifact -> Html AppMsg
completion artifact =
    div [ class "clearfix py1" ]
        [ div [ class "col col-6" ] (viewCompletedPerc artifact)
        , div [ class "col col-6" ] (viewTestedPerc artifact)
        ]


completedPerc : Artifact -> Html msg
completedPerc artifact =
    let
        score =
            completedScore artifact

        color =
            if score >= 3 then
                oliveColor
            else if score >= 2 then
                blueColor
            else if score >= 1 then
                orangeColor
            else
                redColor
    in
        span [ class "bold", colorAttr color ]
            [ text <| (String.left 3 (toString (artifact.completed.spc * 100))) ++ "%" ]


testedPerc : Artifact -> Html msg
testedPerc artifact =
    let
        score =
            testedScore artifact

        color =
            if score >= 2 then
                oliveColor
            else if score >= 1 then
                orangeColor
            else
                redColor
    in
        span [ class "bold", colorAttr color ]
            [ text <| (String.left 3 (toString (artifact.completed.tst * 100))) ++ "%" ]


implemented : Model -> Artifact -> Html AppMsg
implemented model artifact =
    text "FIXME: implemented"
    -- FIXME
    -- div []
    --     (case ( artifact.code, artifact.done ) of
    --         ( Just loc, Nothing ) ->
    --             [ Nav.helpBtn HelpImplemented False
    --             , span [ class "bold" ] [ text "Implemented:" ]
    --             , implementedBasic model artifact
    --             ]

    --         ( Nothing, Just done ) ->
    --             [ Nav.helpBtn HelpDone False
    --             , span [ class "bold" ] [ text "Defined as done:" ]
    --             , implementedBasic model artifact
    --             ]

    --         ( Nothing, Nothing ) ->
    --             [ Nav.helpBtn HelpImplemented False
    --             , span [ class "bold" ] [ text "Implemented:" ]
    --             , implementedBasic model artifact
    --             ]

    --         ( Just _, Just _ ) ->
    --             -- error is displayed by implementedBasic
    --             [ Nav.helpBtn HelpImplemented False
    --             , implementedBasic model artifact
    --             ]
    --     )


{-| show implemented with some settings
-}
implementedBasic : Model -> Artifact -> Html m
implementedBasic model artifact =
    text "FIXME: implementedBasic"
    -- FIXME
    -- let
    --     ( s, d ) =
    --         case ( artifact.code, artifact.done ) of
    --             ( Just loc, Nothing ) ->
    --                 case loc.root of
    --                     Just root ->
    --                         ( [], implementedCodeRoot model root )

    --                     Nothing ->
    --                         ( [], text "sublocations are implemented, see text" )

    --             ( Nothing, Just done ) ->
    --                 ( [], text done )

    --             ( Nothing, Nothing ) ->
    --                 ( [ class "italic gray" ], text "not directly implemented" )

    --             ( Just _, Just _ ) ->
    --                 ( [ class "bold red" ], text "ERROR: code+done both set" )
    -- in
    --     span (s ++ [ getId "implemented" artifact Nothing ]) [ d ]


implementedCodeRoot : Model -> Loc -> Html m
implementedCodeRoot model root =
    let
        plain =
            root.file ++ "[" ++ (toString root.line) ++ "]"
    in
        if model.flags.path_url == "" then
            text plain
        else
            let
                url =
                    Utils.strReplace "{path}" root.file model.flags.path_url
                        |> Utils.strReplace "{line}" (toString root.line)
            in
                a [ href url ] [ text plain ]



-- for the full Edit view


parts : Model -> Artifact -> Html AppMsg
parts model artifact =
    ul
        [ getId "parts" artifact Nothing ]
        (List.map
            (\p ->
                li []
                    [ seeArtifactName model p (ReadChoice artifact) "parts" ]
            )
            artifact.parts
        )


{-| TODO: don't wrap text
-}
textPiece : Model -> Artifact -> Html AppMsg
textPiece model artifact =
    let
        ro =
            model.flags.readonly

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
    if artifact.completed.tst >= 1.0 then
        2
    else if artifact.completed.tst >= 0.5 then
        1
    else
        0


completedScore : Artifact -> Int
completedScore artifact =
    if artifact.completed.spc >= 1.0 then
        3
    else if artifact.completed.spc >= 0.7 then
        2
    else if artifact.completed.spc >= 0.4 then
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
            oliveColor
        else if score >= 3 then
            blueColor
        else if score >= 1 then
            orangeColor
        else
            redColor


{-| colors: olive, blue, orange, red
-}
seeArtifact : Model -> Artifact -> Html AppMsg
seeArtifact model artifact =
    a
        [ class "btn bold"
        , colorAttr <| artifactColor artifact
        , id artifact.name.value
        , onClick (ArtifactsMsg <| ShowArtifact <| artifact.name.value)
        , href (artifactNameUrl artifact.name.value)
        ]
        [ text (artifact.name.raw ++ "  ") ]


{-| TODO: do color and other special stuff for non-existent names
-}
seeArtifactName : Model -> Name -> ViewOption -> String -> Html AppMsg
seeArtifactName model name option attr =
    let
        url =
            artifactNameUrl name.value

        color =
            case getArtifact name.value model of
                Just a ->
                    artifactColor a

                Nothing ->
                    purpleColor
    in
        if memberArtifact name.value model then
            a
                [ class "btn bold"
                , colorAttr color
                , id <| (idFmt attr option) ++ "_" ++ name.value
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


idAttr : String -> ViewOption -> Attribute m
idAttr attr option =
    id <| idFmt attr option


idFmt : String -> ViewOption -> String
idFmt attr option =
    let
        prefix =
            idPrefix option

        name =
            case option of
                ReadChoice artifact ->
                    artifact.name.value

                EditChoice choice ->
                    case choice of
                        ChangeChoice artifact _ ->
                            artifact.name.value

                        CreateChoice _ ->
                            "CREATE"
    in
        prefix ++ attr ++ "_" ++ name


idPrefix : ViewOption -> String
idPrefix option =
    if isRead option then
        "rd_"
    else
        "ed_"
