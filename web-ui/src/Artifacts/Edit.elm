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
import Utils exposing (isJust)

-- regex to search for and replace [[ART-name]]
artifactLinkRegex : Regex.Regex
artifactLinkRegex = 
  Regex.caseInsensitive <| Regex.regex <| "\\[\\[(" ++ artifactValidRaw ++ ")\\]\\]"

view : Model -> Artifact -> Html AppMsg
view model artifact =
  let 
    edit = if isJust artifact.edited && (not model.settings.readonly) then
      [ form model artifact artifact.edited
      -- Header for original view
      , h1 [id "uneditedhead"] [ text "Previous:" ]
      ]
    else
      []
  in
    div []
      ([nav model artifact] 
      ++ edit 
      ++ [form model artifact Nothing])


nav : Model -> Artifact -> Html AppMsg
nav model artifact =
  let
    edit = if model.settings.readonly then
      []
    else if artifact.edited == Nothing then
      [editBtn artifact False]
    else
      [editBtn artifact True
      , saveBtn artifact
      ]
  in
    div 
      [ class "clearfix mb2 white bg-black p1" ]
      (
        [ listBtn ]
        ++ edit
      )


form : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
form model artifact edited =
  div [ class "m3" ]
    [ h1 [getId "ehead" edited] [ text artifact.name.raw ]
    , div [ class "clearfix py1" ]
      [ formColumnOne model artifact
      , formColumnTwo model artifact edited
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
formColumnTwo : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
formColumnTwo model artifact edited =
  div [ class "col col-6" ] 
    [ h3 [] [ text "Text" ]
    , selectRenderedBtns model (isJust edited)
    , displayText model artifact edited
    ]

selectRenderedBtns : Model -> Bool -> Html AppMsg
selectRenderedBtns model editable =
  let
    newView render =
      let
        view = model.state.textView 
      in
        if editable then
          { view | rendered_edit = render }
        else
          { view | rendered_read = render }

    textView = model.state.textView
    (rendered_clr, raw_clr) = if getRendered model editable then
      ("black", "gray")
    else
      ("gray", "black")
  in
    span []
      [ button -- rendered
        [ class ("btn bold " ++ rendered_clr)
        , onClick <| ArtifactsMsg <| ChangeTextViewState <| newView True
        ]
        [ text "rendered" ]
      , button -- raw
        [ class ("btn bold " ++ raw_clr)
        , onClick <| ArtifactsMsg <| ChangeTextViewState <| newView False
        ]
        [ text "raw" ]
      ]

getRendered : Model -> Bool -> Bool
getRendered model edit =
  let
    view = model.state.textView
  in
    if edit then
      view.rendered_edit
    else
      view.rendered_read

displayText : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
displayText model artifact edited =
  if getRendered model (isJust edited) then
    toHtml [] (replaceArtifactLinks model artifact.text)
  else
    displayRawText model artifact edited

-- display raw text in a way that can be edited
displayRawText : Model -> Artifact -> Maybe EditableArtifact -> Html AppMsg
displayRawText model artifact edited =
  let
    editedAttrs = 
      case edited of
        Just e -> 
          [(onInput (\t -> (ArtifactsMsg (EditArtifact artifact.id
              { e | text = t }))))]
        Nothing ->
          []

    attrs = 
      [ class "h3" -- class=h3 otherwise it is really tiny for some reason
      , rows 35
      , cols 80
      , readonly <| not <| isJust edited
      , getId ("text_" ++ artifact.name.value) edited
      ] ++ editedAttrs
    
    rawText = case edited of 
      -- show the edited version
      Just e -> e.text
      -- show the original version
      Nothing -> artifact.text
  in
    textarea attrs [ text rawText ]

listBtn : Html AppMsg
listBtn =
  button
    [ class "btn regular"
    , onClick (ArtifactsMsg ShowArtifacts)
    ]
    [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]

editBtn : Artifact -> Bool -> Html AppMsg
editBtn artifact in_progress =
  button
    [ class "btn regular"
    , if in_progress then
      onClick (ArtifactsMsg (CancelEditArtifact artifact.id))
    else
      onClick (ArtifactsMsg (EditArtifact artifact.id (getEditable artifact)))
    ]
    [ i [ class "fa fa-pencil mr1" ] []
    , text (if in_progress then 
      "Cancel"
    else
      "Edit")
    ]

saveBtn : Artifact -> Html AppMsg
saveBtn artifact =
  button
    [ class "btn regular"
    , onClick <| ArtifactsMsg <| SaveArtifact artifact.id
    ]
    [ i [ class "fa fa-floppy-o mr1" ] []
    , text "Save"
    ]

------------------------
-- Helpers

getId : String -> Maybe EditableArtifact -> Attribute m
getId id_ edited =
  if edited == Nothing then
    id ("rd_" ++ id_) -- read
  else
    id ("ed_" ++ id_) -- edit


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


