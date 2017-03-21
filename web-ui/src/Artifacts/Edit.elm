module Artifacts.Edit exposing (..)

import Html exposing (..)
import Html.Attributes exposing (class, style, value, href, readonly, rows, cols, id)
import Html.Events exposing (onClick, onInput)
import Regex

import Markdown exposing (toHtml)

import Models exposing (Model)
import Routing
import Artifacts.Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Messages exposing (..)
import Artifacts.View as View

-- regex to search for and replace [[ART-name]]
artifactLinkRegex : Regex.Regex
artifactLinkRegex = 
  Regex.caseInsensitive <| Regex.regex <| "\\[\\[(" ++ artifactValidRaw ++ ")\\]\\]"

view : Model -> Artifact -> Html AppMsg
view model artifact =
  div []
    [ nav model
    , form model artifact
    ]

nav : Model -> Html AppMsg
nav model =
  div [ class "clearfix mb2 white bg-black p1" ]
    [ listBtn ]


form : Model -> Artifact -> Html AppMsg
form model artifact =
  div [ class "m3" ]
    [ h1 [id "ehead"] [ text artifact.name.raw ]
    , div [ class "clearfix py1" ]
      [ formColumnOne model artifact
      , formColumnTwo model artifact
      ]
    ]

-- attributes column (non-text)
formColumnOne : Model -> Artifact -> Html AppMsg
formColumnOne model artifact =
  div [ class "col col-6" ]
    [ View.completion artifact
    , View.defined model artifact
    , View.implemented model artifact
    , div [ class "clearfix py1" ] 
      [ div [ class "col col-6" ] 
        [ h3 [] [ text "Partof" ]
        , View.partof model artifact
        ]
      , div [ class "col col-6" ] 
        [ h3 [] [ text "Parts" ]
        , View.parts model artifact
        ]
      ]
    ]

-- Text column
formColumnTwo : Model -> Artifact -> Html AppMsg
formColumnTwo model artifact =
  div [ class "col col-6" ] 
    [ h3 [] [ text "Text" ]
    , selectRenderedBtns model
    , displayText model artifact
    ]

selectRenderedBtns : Model -> Html AppMsg
selectRenderedBtns model =
  let
    edit = model.state.edit
    (rendered_clr, raw_clr) = if edit.rendered then
      ("black", "gray")
    else
      ("gray", "black")
  in
    span []
      [ button -- rendered
        [ class ("btn bold " ++ rendered_clr)
        , onClick <| ArtifactsMsg <| EditStateChanged { rendered = True }
        ]
        [ text "rendered" ]
      , button -- raw
        [ class ("btn bold " ++ raw_clr)
        , onClick <| ArtifactsMsg <| EditStateChanged { rendered = False }
        ]
        [ text "raw" ]
      ]

displayText : Model -> Artifact -> Html AppMsg
displayText model artifact =
  if model.state.edit.rendered then
    toHtml [] (replaceArtifactLinks model artifact.text)
  else
    displayRawText model artifact

displayRawText : Model -> Artifact -> Html AppMsg
displayRawText model artifact =
  let
    edited = getEdited artifact
  in
    textarea 
      [ class "h3" -- class=h3 otherwise it is really tiny for some reason
      , rows 35
      , cols 80
      , readonly model.settings.readonly 
      , id ("text_" ++ artifact.name.value)
      , onInput (\t -> (ArtifactsMsg (ArtifactEdited artifact.name.value 
        { edited | text = t })))
      ] 
      [ text artifact.text ]

listBtn : Html AppMsg
listBtn =
  button
    [ class "btn regular"
    , onClick (ArtifactsMsg ShowArtifacts)
    ]
    [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]

-- get the full url to a single artifact
fullArtifactUrl : Model -> String -> String
fullArtifactUrl model indexName =
  let
    addrName = String.toLower (indexNameUnchecked indexName)
    -- super hacky way to get the origin: might fail for files
    -- I tried location.origin... doesn't work for some reason.
    -- neither does location.host + location.pathname
    origin = case List.head (String.split "#" model.location.href) of
      Just o -> removeSlashEnd o
      Nothing -> "ERROR-origin-no-head"
  in
    origin ++ "/" ++ artifactsUrl ++ "/" ++ addrName

removeSlashEnd : String -> String
removeSlashEnd path =
  if String.endsWith "/" path then
    removeSlashEnd (String.dropRight 1 path)
  else
    path

-- replace [[ART-name]] with [ART-name](link)
replaceArtifactLinks : Model -> String -> String
replaceArtifactLinks model text =
  let
    replace : Regex.Match -> String
    replace match =
      case List.head match.submatches of
        Just m -> case m of
          Just m -> "[" ++ m ++ "](" ++ (fullArtifactUrl model m) ++ ")"
          Nothing -> "INTERNAL_ERROR"
        Nothing -> "INTERNAL_ERROR"
  in
    Regex.replace Regex.All artifactLinkRegex replace text
