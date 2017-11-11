module Artifacts.PartGraph exposing (..)

import Models exposing (Model, getArtifact)
import Artifacts.View exposing (..)
import Artifacts.Models exposing (..)


-- DOT builders


renderPart : Model -> Artifact -> String
renderPart model artifact =
    let
        getRaw =
            (\n -> n.raw)

        parts =
            List.map getRaw artifact.parts

        partof =
            List.map getRaw artifact.partof

        settings =
            List.concat
                [ [ nodeSettings model artifact True ]
                , List.map (\n -> nameNodeSettings model n) parts
                , List.map (\n -> nameNodeSettings model n) partof
                ]

        connections =
            List.concat
                [ List.map (\pof -> connectNames pof artifact.name.value) partof
                , List.map (\pt -> connectNames artifact.name.value pt) parts
                ]

        lines =
            List.concat [ settings, connections ]
    in
        wrapDot (String.join "\n" lines)


wrapDot : String -> String
wrapDot dot =
    String.join "\n"
        [ "```dot\ndigraph G {"
        , "graph [rankdir=LR, fontsize=14, margin=0.001];"
        , "subgraph cluster_family { margin=4; label=<<b>related artifacts</b>>"
        , dot
        , "}}\n```\n"
        ]


connectNames : String -> String -> String
connectNames from to =
    ("\""
        ++ (indexNameUnchecked from)
        ++ "\" -> \""
        ++ (indexNameUnchecked to)
        ++ "\""
    )


{-| Given a name always get it's node's settings.
-}
nameNodeSettings : Model -> String -> String
nameNodeSettings model rawName =
    case getArtifact rawName model of
        Just artifact ->
            nodeSettings model artifact False

        Nothing ->
            dneNodeSettings rawName


{-| Render a node name that does not exist.
-}
dneNodeSettings : String -> String
dneNodeSettings name =
    let
        indexName =
            indexNameUnchecked name
    in
        String.join " "
            [ "{\"" ++ indexName ++ "\" ["
            , "label=<<i>" ++ name ++ "</i>>;"
            , "fontcolor=black; style=filled; fillcolor=pink;"
            , "fontsize=12; margin=0.01; shape=invhouse;"
            , "tooltip=\"Name not found\";"
            , "]}"
            ]


{-| Get the "node settings" graph item
-}
nodeSettings : Model -> Artifact -> Bool -> String
nodeSettings model artifact is_focus =
    let
        attrs =
            if is_focus then
                "penwidth=1.5; style=filled; fillcolor=cyan;"
            else
                ""
    in
        String.join " "
            [ "{\"" ++ artifact.name.value ++ "\" ["
            , "label=<<b>" ++ artifact.name.raw ++ "</b>>;"
            , "href=\"" ++ (artifactNameUrl artifact.name.value) ++ "\";"
            , "fontcolor=\"" ++ (artifactColor artifact) ++ "\";"
            , "fontsize=12; margin=0.01; shape=invhouse;"
            , attrs
            , "]}"
            ]
