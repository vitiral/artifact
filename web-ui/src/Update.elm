module Update exposing (..)

import Dict
import Navigation
import Log
import Utils
import Artifacts.Models exposing (ViewOption(..), EditOption(..))
import Artifacts.Update
import Artifacts.TextLinks exposing (replaceArtifactLinks)
import Artifacts.PartGraph exposing (renderPart)
import Messages exposing (AppMsg(..), formatHttpError, helpUrl, helpRepr, checkUrl)
import Models
    exposing
        ( Model
        , RenderedText
        , LogMsg(..)
        , getViewingArtifact
        , ViewingArtifact(..)
        , getEditViewOption
        )
import Ports


update : AppMsg -> Model -> ( Model, Cmd AppMsg )
update msg model =
    case msg of
        ArtifactsMsg subMsg ->
            let
                ( new_model, new_cmd ) =
                    Artifacts.Update.update subMsg model
            in
                requestRerender new_model [ new_cmd ]

        AckLogMsg index ->
            let
                ( _, logs ) =
                    Utils.popIndexUnsafe index model.logs
            in
                ( { model | logs = logs }, Cmd.none )

        RouteChange route ->
            let
                new_model =
                    { model | route = route }
            in
                requestRerender new_model []

        HttpError err ->
            ( Log.log model <| LogErr <| formatHttpError err, Cmd.none )

        AppError err ->
            ( Log.log model <| LogErr <| "AppError: " ++ err, Cmd.none )

        ShowHelp page ->
            let
                url =
                    "#" ++ helpUrl ++ "/" ++ (helpRepr page)
            in
                ( model, Navigation.newUrl url )

        ShowCheck ->
            ( model, Navigation.newUrl <| "#" ++ checkUrl )

        RenderText text ->
            let
                _ =
                    Debug.log "INVALID request to render artifacts" text
            in
                ( model, Cmd.none )

        TextRendered ( text, part ) ->
            let
                rendered : RenderedText
                rendered =
                    { text = text, part = part }
            in
                ( { model | rendered = Just rendered }, Cmd.none )

        Noop ->
            ( model, Cmd.none )


requestRerender : Model -> List (Cmd AppMsg) -> ( Model, Cmd AppMsg )
requestRerender model cmds =
    let
        -- Make a call to get the text rendered AND invalidate
        -- the existing rendered text
        final_model =
            { model | rendered = Nothing }

        renderCmds =
            case getViewingUnrendered model of
                Just unr ->
                    let
                        text =
                            replaceArtifactLinks model unr.text
                    in
                        [ Ports.renderText ( text, unr.part ) ]

                Nothing ->
                    []

        final_cmds =
            List.append renderCmds cmds
    in
        ( final_model, Cmd.batch final_cmds )


{-| Helper function to get the unrendered text of the artifact that is
currently being viewed.
-}
getViewingUnrendered : Model -> Maybe { text : String, part : String }
getViewingUnrendered model =
    -- Note: Cannot render `part` in editable since we don't know its `parts`
    case getViewingArtifact model of
        ViewingExist id ->
            case Dict.get id model.artifacts of
                Nothing ->
                    Nothing

                Just art ->
                    case getEditViewOption model art of
                        ReadChoice art ->
                            Just
                                { text = art.text
                                , part = renderPart model art
                                }

                        EditChoice option ->
                            case option of
                                ChangeChoice _ editable ->
                                    Just
                                        { text = editable.text
                                        , part = ""
                                        }

                                CreateChoice editable ->
                                    -- TODO: I think this is impossible
                                    Just
                                        { text = editable.text
                                        , part = ""
                                        }

        ViewingCreate ->
            case model.create of
                Just editable ->
                    Just
                        { text = editable.text
                        , part = ""
                        }

                Nothing ->
                    Nothing

        ViewingError _ ->
            Nothing

        ViewingNothing ->
            Nothing
