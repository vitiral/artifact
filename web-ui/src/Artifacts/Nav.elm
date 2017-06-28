module Artifacts.Nav exposing (..)

-- The Nav module is the only interface for moving throughout the app
-- and initiating commands to the api server. The Update module actually sends
-- them, but the messages are only created here

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick)
import Utils exposing (..)
import Models exposing (..)
import Messages exposing (AppMsg(..))
import Artifacts.Models exposing (..)
import Artifacts.Messages exposing (..)


-- NAV BAR


bar : List (Html AppMsg) -> Html AppMsg
bar elements =
    div
        [ class "clearfix mb2 white bg-black p1" ]
        elements


{-| nav bar for list view
-}
listBar : List (Html AppMsg)
listBar =
    [ div [ class "left p2" ] [ text "Artifacts" ]
    ]


{-| nav bar for read-only artifact view
-}
readBar : Model -> Artifact -> List (Html AppMsg)
readBar model artifact =
    [ listBtn ]


{-| nav bar for edit artifact view
-}
editBar : Model -> Artifact -> List (Html AppMsg)
editBar model artifact =
    let
        extra =
            case artifact.edited of
                Just e ->
                    [ editBtn artifact True
                    , saveBtn model artifact e
                    ]

                Nothing ->
                    [ editBtn artifact False ]
    in
        [ listBtn ] ++ extra



-- ACTIONS


{-| navigate back to the list page
-}
listBtn : Html AppMsg
listBtn =
    button
        [ class "btn regular"
        , id "list"
        , onClick (ArtifactsMsg ShowArtifacts)
        ]
        [ i [ class "fa fa-chevron-left mr1" ] [], text "List" ]


{-| start/stop editing
-}
editBtn : Artifact -> Bool -> Html AppMsg
editBtn artifact in_progress =
    button
        ([ class "btn regular"
         ]
            ++ if in_progress then
                [ id "cancel_edit"
                , onClick (ArtifactsMsg (CancelEditArtifact artifact.id))
                ]
               else
                [ id "edit"
                , onClick (ArtifactsMsg (EditArtifact artifact.id (getEditable artifact)))
                ]
        )
        [ i [ class "fa fa-pencil mr1" ] []
        , text
            (if in_progress then
                "Cancel"
             else
                "Edit"
            )
        ]


{-| save the current edit state. This button does not always exist.
-}
saveBtn : Model -> Artifact -> EditableArtifact -> Html AppMsg
saveBtn model artifact edited =
    let
        ( t, color, d ) =
            case checkFull model artifact edited of
                True ->
                    ( "save artifact", "", False )

                False ->
                    ( "cannot save: errors exist", " red ", True )
    in
        button
            [ class <| "btn regular" ++ color
            , id "save"
            , title t
            , onClick <| ArtifactsMsg <| SaveArtifact artifact.id
            , disabled d
            ]
            [ i [ class "fa fa-floppy-o mr1" ] []
            , text "Save"
            ]



-- CHECKS


{-| return False if the editable piece is not valid
-}
checkFull : Model -> Artifact -> EditableArtifact -> Bool
checkFull model artifact edited =
    let
        -- FIXME: needs to accept option
        ch_name =
            isOk <| checkName model edited.name (ChangeChoice artifact edited) 

        ch_partof =
            List.map (checkPartof model edited.name) edited.partof
                |> List.all isOk
    in
        ch_name && ch_partof


{-| Just check that the name is valid and that it doesn't
already exist.
-}
checkName : Model -> String -> EditOption -> Result String Name
checkName model name option =
    case initName name of
        Ok name ->
            case option of
                ChangeChoice artifact _ ->
                    if name == artifact.name then
                        -- name already exists... because its the same name!
                        Ok name
                    else
                        checkNameSimple model name
                CreateChoice _ ->
                    checkNameSimple model name
        Err _ ->
            Err "invalid name"


checkNameSimple : Model -> Name -> Result String Name
checkNameSimple model name =
    if memberArtifact name.value model then
        Err "name already exists"
    else
        Ok name

{-| return some error if the name cannot be a partof `partof`
(i.e. if `partof` cannot be in name's partof attrs)

Possible errors:

  - name is invalid
  - partof does not exist
  - partof/name are invalid types

Otherwise return the valid name

-}
checkPartof : Model -> String -> Name -> Result String Name
checkPartof model name partof =
    case initName name of
        Ok name ->
            if not <| memberArtifact partof.value model then
                Err "does not exist"
            else if not <| validPartof name partof then
                Err "invalid type"
            else
                Ok name

        Err _ ->
            Err "invalid artifact name"
