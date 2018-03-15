module Artifacts.Select exposing (..)

-- module to define `select` elements

import Dict
import Set
import Html exposing (..)
import Html.Attributes exposing (value, class, href, title, id, selected)
import Html.Events exposing (onClick, on, targetValue)
import Json.Decode as Json
import Styles exposing (warning)
import Messages exposing (AppMsg(..), Route(..), HelpPage(..))
import Models exposing (Model, getArtifact, memberArtifact, getFiles)
import Utils exposing (..)
import Artifacts.Models exposing (..)
import Artifacts.Nav as Nav
import Artifacts.Messages exposing (Msg(..))
import Artifacts.View as View


{-| onChange is the correct event to use for select boxes
-}
onChange : (String -> msg) -> Attribute msg
onChange msg =
    on "change" <| Json.map msg targetValue


{-| view or edit the partof fields
-}
partof : Model -> ViewOption -> Html AppMsg
partof model option =
    let
        ( values, addPartof, getItem ) =
            case option of
                ReadChoice artifact ->
                    let
                        poItem ( _, name ) =
                            li []
                                [ View.seeArtifactName model name option "partof" ]

                        add =
                            []
                    in
                        ( artifact.partof, add, poItem )

                EditChoice choice ->
                    let
                        edited =
                            getEdited choice

                        ( a, b ) =
                            editPartofComponents model choice
                    in
                        ( edited.partof, a, b )
    in
        ul
            [ View.idAttr "partof" option ]
            ((List.map getItem (enumerate values))
                ++ addPartof
            )


{-| return components necessary for editing partof

returns:

  - addPartof: a list containing the dropdown for creating new partof pieces
  - getItem: given a (index, name) it has the logic for selecting (editing) a
    partof item

-}
editPartofComponents :
    Model
    -> EditOption
    -> ( List (Html AppMsg), ( Int, Name ) -> Html AppMsg )
editPartofComponents model option =
    let
        edited =
            getEdited option

        -- return names that this name could be a partof
        validPartofs model name =
            case initName name of
                Ok name ->
                    let
                        names =
                            List.map (\a -> a.name) (Dict.values model.artifacts)

                        out =
                            List.filter (\n -> validPartof name n) names
                    in
                        List.sortBy (\n -> n.value) out

                Err _ ->
                    []

        valid =
            validPartofs model edited.name

        getItem ( index, name ) =
            li
                []
                (partofItem model valid option index name)

        emptyName : Name
        emptyName =
            { raw = "---"
            , value = "---"
            , ty = Req
            }

        createMsg rawName =
            if rawName == emptyName.value then
                Noop
            else
                let
                    partof =
                        List.sortBy (\n -> n.value) <|
                            edited.partof
                                ++ [ initNameUnsafe rawName ]
                in
                    ArtifactsMsg <|
                        EditArtifact <|
                            setEdited
                                option
                                { edited | partof = partof }

        -- you can only add elements that aren't already present
        partof_exist =
            Set.fromList <| List.map (\n -> n.value) edited.partof

        create_valid =
            List.filter (\n -> not <| Set.member n.value partof_exist) valid

        add_partof =
            [ i [ class "fa fa-plus-square mr1 olive", title "add partof" ] []
            , selectPartof
                model
                ([ emptyName ] ++ create_valid)
                emptyName
                ("add_partof_" ++ (getNameIndex option))
                createMsg
            ]
    in
        ( add_partof, getItem )


{-| return the full html element of a partof item

This includes buisness logic for:

  - deleting the item
  - changing the item's value to valid names ONLY

In addition, it also handles the following cases:

  - if the item was created automatically, it is read-only
  - if the item is invalid, it is delete-only (not editable)

-}
partofItem :
    Model
    -> List Name
    -> EditOption
    -> Int
    -> Name
    -> List (Html AppMsg)
partofItem model valid_partofs option index name =
    let
        parent_name =
            (getNameIndex option)

        edited =
            getEdited option

        part_id =
            parent_name ++ "_" ++ name.value

        deleteMsg =
            let
                partof =
                    Tuple.second <| popIndexUnsafe index edited.partof
            in
                ArtifactsMsg <| EditArtifact <| setEdited option { edited | partof = partof }

        deleteBtn =
            button
                [ class "btn bold red"
                , id <| "delete_partof_" ++ part_id
                , onClick deleteMsg
                , title "delete"
                ]
                [ i [ class "fa fa-trash" ] [] ]

        updateMsg rawName =
            let
                updateName =
                    initNameUnsafe rawName

                partof =
                    List.sortBy (\n -> n.value) <|
                        setIndexUnsafe index updateName edited.partof
            in
                ArtifactsMsg <|
                    EditArtifact <|
                        setEdited
                            option
                            { edited | partof = partof }

        see_art =
            View.seeArtifactName model name (EditChoice option) "partof"
    in
        case Nav.checkPartof model edited.name name of
            Err error ->
                -- it is invalid. Only allow deletion
                [ deleteBtn
                , see_art
                , warning error
                ]

            Ok ed_name ->
                if autoPartof ed_name name then
                    -- it was created automatically and CANNOT be edited
                    [ button
                        [ class "btn bold"
                        , title "created automatically"
                        ]
                        [ i [ class "fa fa-info" ] [] ]
                    , see_art
                    ]
                else
                    -- it is editable
                    let
                        -- cannot set to other partofs that exist
                        -- or to own current/future name
                        partof_exist =
                            Set.remove name.value <|
                                Set.union (Set.fromList [ ed_name.value, parent_name ]) <|
                                    Set.fromList (List.map (\n -> n.value) edited.partof)

                        valid =
                            List.filter
                                (\n ->
                                    not <|
                                        Set.member n.value partof_exist
                                )
                                valid_partofs
                    in
                        [ deleteBtn
                        , see_art
                        , selectPartof model valid name ("select_partof_" ++ part_id) updateMsg
                        ]


{-| the select html element
-}
selectPartof :
    Model
    -> List Name
    -> Name
    -> String
    -> (String -> AppMsg)
    -> Html AppMsg
selectPartof model valid_partofs name id_ selectMsg =
    select
        [ class "select-tiny"
        , id id_
        , onChange selectMsg
        ]
        (List.map
            (\n ->
                option
                    [ id <| "select_option_" ++ name.value ++ "_" ++ n.value
                    , value n.raw
                    , selected (n == name)
                    ]
                    [ text <| "  " ++ n.raw ]
            )
            valid_partofs
        )



-- DEFINED ELEMENT


{-| shows the location where the artifact is defined
(which toml file it is written in)
-}
defined : Model -> ViewOption -> Html AppMsg
defined model option =
    let
        element =
            case option of
                ReadChoice artifact ->
                    let
                        url =
                            strReplace "{path}" artifact.file model.flags.path_url
                    in
                        span
                            [ View.idAttr "def" option ]
                            [ a [ href url ] [ text artifact.file ] ]

                EditChoice choice ->
                    editDefined model choice
    in
        div []
            [ span [ class "bold" ]
                [ Nav.helpBtn HelpDefined False
                , text "Defined at:"
                ]
            , element
            ]


editDefined : Model -> EditOption -> Html AppMsg
editDefined model ed_option =
    let
        edited =
            getEdited ed_option

        selectMsg file =
            ArtifactsMsg <| EditArtifact <| setEdited ed_option { edited | file = file }

        warn =
            case Nav.checkDef model edited of
                Ok _ ->
                    []

                Err e ->
                    [ warning e ]

        elements =
            [ [ span [ class "bold" ] [ text edited.file ]
              , select
                    [ class "select-tiny"
                    , View.idAttr "file" (EditChoice ed_option)
                    , onChange selectMsg
                    ]
                    (List.map
                        (\d ->
                            option
                                [ value d
                                , selected (d == edited.file)
                                ]
                                [ text <| "  " ++ d ]
                        )
                        (getFiles model <| Just edited.file)
                    )
              ]
            , warn
            ]
    in
        span [] <| List.concat elements
