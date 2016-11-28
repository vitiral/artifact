module Artifacts.View exposing (..)
{-
Generic view methods that can be used in multiple places (List, Edit, etc)
-}

import String
import Html exposing (..)
import Html.Attributes exposing (class, href, title)
import Html.Events exposing (onClick)
import Navigation

import Messages exposing (AppMsg(..), Route(..))
import Models exposing (Model)
import Artifacts.Models exposing (Artifact, artifactNameUrl, realName)
import Artifacts.Messages exposing (Msg(..))

completion : Artifact -> Html AppMsg
completion artifact =
  div [ class "clearfix py1" ]
    [ div [ class "col col-6" ] 
      [ span [class "bold" ] [ text "Completed: " ]
      , completedPerc artifact
      ]
    , div [ class "col col-6" ] 
      [ span [class "bold" ] [ text "Tested: " ]
      , testedPerc artifact
      ]
    ]

completedPerc artifact =
  text <| (String.left 3 (toString (artifact.completed * 100))) ++ "%"

testedPerc artifact =
  text <| (String.left 3 (toString (artifact.tested * 100))) ++ "%"

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
      span [class "italic" ] [ text "not directly implemented" ])

parts : Model -> Artifact -> Html AppMsg
parts model artifact =
  ul [] (List.map (\p -> li [ class "underline" ] [ seeArtifactName model p ]) artifact.parts)


-- TODO: allow editing when not readonly
partof : Model -> Artifact -> Html AppMsg
partof model artifact =
  ul [] (List.map (\p -> li [ class "underline" ] [ seeArtifactName model p ]) artifact.partof)

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

seeArtifact : Model -> Artifact -> Html AppMsg
seeArtifact model artifact =
  let
    ro = model.settings.readonly
  in
    button 
      [ class "btn bold"
      , onClick (ArtifactsMsg <| ShowArtifact <| artifact.name)
      , href (artifactNameUrl artifact.name)
      ]
      [ text (artifact.raw_name ++ "  ")
      , i [ class <| if ro then "bold fa fa-eye mr1" else "bold fa fa-pencil mr1" 
        , href (artifactNameUrl artifact.name) 
        ] []
      
      ]

-- TODO: do color and other special stuff for non-existent names
seeArtifactName : Model -> String -> Html AppMsg
seeArtifactName model name =
  let
    hasName = \a -> a.name == (realName name)
    exists = case List.head <| List.filter hasName model.artifacts of
      Just _ -> True
      Nothing -> False

    url = (artifactNameUrl name)
  in 
    if exists then
      span 
        [ href url
        , onClick ( RouteChange <| ArtifactNameRoute <| realName name ) 
        ] [ text name ]
    else
      span [ title "Name not found" ] [ text name ]
