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
import Models exposing (Settings)
import Artifacts.Models exposing (Artifact, artifactUrl, artifactNameUrl)
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
  text (toString (artifact.completed * 100) ++ "%")

testedPerc artifact =
  text (toString (artifact.tested * 100) ++ "%")

defined : Settings -> Artifact -> Html AppMsg
defined settings artifact =
  div [] 
  [ span [class "bold" ] [ text "Defined at: " ]
  , text artifact.path
  ]

implemented : Settings -> Artifact -> Html m
implemented settings artifact =
  div [] 
    (case artifact.loc of
      Just loc ->
        [ span [class "bold" ] [ text "Implemented at: " ]
        , implementedBasic settings artifact
        ]
      Nothing ->
        []
    )

implementedBasic : Settings -> Artifact -> Html m
implementedBasic settings artifact = 
  (case artifact.loc of 
    Just loc ->
      text (loc.path ++ " (" ++ (toString loc.row) 
            ++ "," ++ (toString loc.col) ++ ")"
           )
    Nothing ->
      span [class "italic" ] [ text "not directly implemented" ])

parts : Settings -> Artifact -> Html AppMsg
parts settings artifact =
  ul [] (List.map (\p -> li [ class "underline" ] [ seeArtifactName p ]) artifact.parts)


-- TODO: allow editing when not readonly
partof : Settings -> Artifact -> Html AppMsg
partof settings artifact =
  ul [] (List.map (\p -> li [ class "underline" ] [ seeArtifactName p ]) artifact.partof)

textPiece : Settings -> Artifact -> Html AppMsg
textPiece settings artifact =
  let
    ro = settings.readonly
    text_part = String.left 200 artifact.text
    t = if (String.length artifact.text) > 200 then
      text_part ++ " ..."
    else
      text_part
  in
    text text_part

seeArtifact : Settings -> Artifact -> Html AppMsg
seeArtifact settings artifact =
  let
    ro = settings.readonly
  in
    button 
      [ class "btn bold"
      , onClick (ArtifactsMsg <| ShowArtifact artifact.id)
      , href (artifactUrl artifact.id)
      ]
      [ text (artifact.name ++ "  ")
      , i [ class <| if ro then "bold fa fa-eye mr1" else "bold fa fa-pencil mr1" 
        , href (artifactUrl artifact.id) 
        ] []
      
      ]

seeArtifactName : String -> Html AppMsg
seeArtifactName name =
  let
    url = (artifactNameUrl name)
  in 
    span 
      [ href url
      , onClick ( RouteChange (ArtifactNameRoute name) ) 
      ] [ text name ]
 
