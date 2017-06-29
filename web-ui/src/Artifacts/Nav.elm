module Artifacts.Nav exposing (..)

-- The Nav module is the only interface for moving throughout the app
-- and initiating commands to the api server. The Update module actually sends
-- them, but the messages are only created here

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick)
import Utils exposing (..)
import Models exposing (..)
import Log
import Messages exposing (AppMsg(..))
import Artifacts.Models exposing (..)
import Artifacts.Messages exposing (..)


-- NAV BAR


bar : Model -> List (Html AppMsg) -> Html AppMsg
bar model elements =
    div []
        [ div
            [ class "clearfix mb2 white bg-black p1" ]
            elements
        , Log.view model
        ]


{-| nav bar for list view
-}
listBar : List (Html AppMsg)
listBar =
    [ div [ class "left p2" ] [ text "Artifacts" ]
    , createBtn
    ]


{-| nav bar for read-only artifact view
-}
readBar : List (Html AppMsg)
readBar =
    [ listBtn ]


{-| nav bar for edit artifact view
-}
editBar : Model -> ViewOption -> List (Html AppMsg)
editBar model option =
    let
        extra =
            case option of
                ReadChoice artifact ->
                    [ createBtn
                    , deleteBtn artifact
                    , editBtn option
                    ]

                EditChoice choice ->
                    case choice of
                        ChangeChoice artifact edited ->
                            [ createBtn
                            , deleteBtn artifact
                            , editBtn option
                            , saveBtn model choice
                            ]

                        CreateChoice edited ->
                            [ editBtn option
                            , saveBtn model choice
                            ]
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
        , onClick <| ArtifactsMsg ShowArtifacts
        ]
        [ text "List" ]


{-| navigate to the create page
-}
createBtn : Html AppMsg
createBtn =
    button
        [ class "btn regular"
        , id "create"
        , onClick <| ArtifactsMsg CreateArtifact
        ]
        [ i [ class "fa fa-plus-square mr1" ] []
        , text "Create New"
        ]


{-| start/stop editing
-}
editBtn : ViewOption -> Html AppMsg
editBtn option =
    let
        ( t, attrs ) =
            case option of
                ReadChoice artifact ->
                    -- editing has not yet started
                    ( "Edit"
                    , [ id "edit"
                      , onClick <|
                            ArtifactsMsg <|
                                EditArtifact <|
                                    ChangeChoice artifact (getEditable artifact)
                      ]
                    )

                EditChoice choice ->
                    ( "Cancel"
                    , [ id "cancel_edit"
                      , onClick <| ArtifactsMsg <| CancelEditArtifact choice
                      ]
                    )
    in
        button
            ([ class "btn regular" ] ++ attrs)
            [ i [ class "fa fa-pencil mr1" ] []
            , text t
            ]


{-| save the current edit state. This button does not always exist.
-}
saveBtn : Model -> EditOption -> Html AppMsg
saveBtn model option =
    let
        ( t, color, d ) =
            case checkFull model option of
                True ->
                    ( "save artifact", "", False )

                False ->
                    ( "cannot save: errors exist", " red ", True )
    in
        button
            [ class <| "btn regular" ++ color
            , id "save"
            , title t
            , onClick <| ArtifactsMsg <| SaveArtifact option
            , disabled d
            ]
            [ i [ class "fa fa-floppy-o mr1" ] []
            , text "Save"
            ]


deleteBtn : Artifact -> Html AppMsg
deleteBtn artifact =
    button
        [ class "btn regular"
        , id "delete"
        , title "delete artifact"
        , onClick <| ArtifactsMsg <| DeleteArtifact artifact
        ]
        [ i [ class "fa fa-trash mr1" ] []
        , text "Delete"
        ]



-- CHECKS


{-| return False if the editable piece is not valid
-}
checkFull : Model -> EditOption -> Bool
checkFull model option =
    let
        ch_name =
            isOk <| checkName model edited.name option

        edited =
            getEdited option

        ch_partof =
            List.map (checkPartof model edited.name) edited.partof
                |> List.all isOk

        ch_def =
            isOk <| checkDef model edited
    in
        ch_name && ch_partof && ch_def


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


checkDef : Model -> EditableArtifact -> Result String String
checkDef model edited =
    if List.member edited.def (getDefs model Nothing) then
        Ok edited.def
    else
        Err "invalid definition path"
