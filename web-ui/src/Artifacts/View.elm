module Artifacts.View exposing (..)
{-
Generic view methods that can be used in multiple places (List, Edit, etc)
-}

import String
import Dict
import Html exposing (..)
import Html.Attributes exposing (class, href, title, id)
import Html.Events exposing (onClick)

import Messages exposing (AppMsg(..), Route(..))
import Models exposing (Model)
import Artifacts.Models exposing (Artifact, artifactNameUrl, indexName, indexNameUnchecked)
import Artifacts.Messages exposing (Msg(..))

completion : Artifact -> Html AppMsg
completion artifact =
  div [ class "clearfix py1" ]
    [ div [ class "col col-6" ] 
      [ span [class "bold" ] [ text "Implemented: " ]
      , completedPerc artifact
      ]
    , div [ class "col col-6" ] 
      [ span [class "bold" ] [ text "Tested: " ]
      , testedPerc artifact
      ]
    ]

completedPerc : Artifact -> Html msg
completedPerc artifact =
  let
    score = completedScore artifact

    color = if score >= 3 then
      "olive"
    else if score >= 2 then
      "blue"
    else if score >= 1 then
      "orange"
    else
      "red"
  in
    span [ class ("bold " ++color) ] 
      [ text <| (String.left 3 (toString (artifact.completed * 100))) ++ "%" ]

testedPerc : Artifact -> Html msg
testedPerc artifact =
  let
    score = testedScore artifact

    color = if score >= 2 then
      "olive"
    else if score >= 1 then
      "orange"
    else
      "red"
  in
    span [ class ("bold " ++ color) ] 
      [ text <| (String.left 3 (toString (artifact.tested * 100))) ++ "%" ]

defined : Model -> Artifact -> Html AppMsg
defined model artifact =
  div [] 
  [ span [class "bold" ] [ text "Defined at: " ]
  , text artifact.path
  ]

implemented : Model -> Artifact -> Html m
implemented model artifact =
  div [] 
    (case artifact.loc of
      Just loc ->
        [ span [class "bold" ] [ text "Implemented at: " ]
        , implementedBasic model artifact
        ]
      Nothing ->
        []
    )

implementedBasic : Model -> Artifact -> Html m
implementedBasic model artifact = 
  (case artifact.loc of 
    Just loc ->
      text (loc.path ++ " (" ++ (toString loc.row) 
            ++ "," ++ (toString loc.col) ++ ")"
           )
    Nothing ->
      span [class "italic gray" ] [ text "not directly implemented" ])

parts : Model -> Artifact -> Html AppMsg
parts model artifact =
  let
    rawParts = List.map (\p -> p.raw) artifact.parts
  in
    ul
      [ id ("parts_" ++ artifact.name.value) ] 
      (List.map (\p -> li [ class "underline" ] [ seeArtifactName model p ]) rawParts)


-- TODO: allow editing when not readonly
partof : Model -> Artifact -> Html AppMsg
partof model artifact =
  let
    rawPartof = List.map (\p -> p.raw) artifact.partof
  in
    ul 
      [ id ("partof_" ++ artifact.name.value) ] 
      (List.map (\p -> li [ class "underline" ] [ seeArtifactName model p ]) rawPartof)

-- TODO: don't wrap text
textPiece : Model -> Artifact -> Html AppMsg
textPiece model artifact =
  let
    ro = model.settings.readonly
    text_part = String.left 200 artifact.text
    t = if (String.length artifact.text) > 200 then
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
    score = (testedScore artifact) + (completedScore artifact)
  in
    if score >= 5 then
      "olive"
    else if score >= 3 then
      "blue"
    else if score >= 1 then
      "orange"
    else
      "red"

-- colors: olive, blue, orange, red
seeArtifact : Model -> Artifact -> Html AppMsg
seeArtifact model artifact =
  a 
    [ class ("btn bold " ++ (artifactColor artifact))
    , onClick (ArtifactsMsg <| ShowArtifact <| artifact.name.value)
    , href (artifactNameUrl artifact.name.value)
    , id artifact.name.value
    ]
    [ text (artifact.name.raw ++ "  ") ]

-- TODO: do color and other special stuff for non-existent names
seeArtifactName : Model -> String -> Html AppMsg
seeArtifactName model name =
  let
    indexName = indexNameUnchecked name

    url = (artifactNameUrl indexName)

    color = case Dict.get indexName model.artifacts of
      Just a -> artifactColor a
      Nothing -> "purple"
  in 
    if Dict.member indexName model.artifacts then
      a 
        [ class ("btn bold " ++ color)
        , href url
        , onClick ( ArtifactsMsg <| ShowArtifact <| indexName ) 
        ] [ text name ]
    else
      span [ class ("btn " ++ color) ] [ text name ]
