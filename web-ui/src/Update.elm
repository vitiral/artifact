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
import Delay
import Debounce
import Time


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

        -- debounced render request
        RequestRender db_msg ->
            case getViewingUnrendered model of
                Nothing ->
                    ( model, Cmd.none )

                Just unr ->
                    let
                        text =
                            replaceArtifactLinks model unr.text

                        render =
                            (\_ -> Ports.renderText ( text, unr.part ))

                        ( debounce, cmd ) =
                            Debounce.update
                                debounceConfig
                                (Debounce.takeLast render)
                                db_msg
                                model.debounceRender
                    in
                        ( { model | debounceRender = debounce }, cmd )

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
                ( { model | rendered = Just rendered }
                , Delay.after 250 Time.millisecond UnlockRender
                )

        UnlockRender ->
            ( model, Debounce.unlock debounceConfig )

        Noop ->
            ( model, Cmd.none )


debounceConfig : Debounce.Config AppMsg
debounceConfig =
    -- Unfortunately this waits for 100 seconds after the FIRST request.
    -- I think I want to wait 100 seconds after I currently unlock, which
    -- probably requires a completely new message/setting
    { strategy = Debounce.manual
    , transform = RequestRender
    }


requestRerender : Model -> List (Cmd AppMsg) -> ( Model, Cmd AppMsg )
requestRerender model cmds =
    let
        ( debounce, cmd ) =
            Debounce.push debounceConfig () model.debounceRender
    in
        ( { model
            | rendered = Nothing
            , debounceRender = debounce
          }
        , Cmd.batch <| [ cmd ] ++ cmds
        )


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
