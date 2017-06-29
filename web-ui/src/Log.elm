module Log exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick)
import Models exposing (..)
import Messages exposing (..)
import Utils


-- VIEW


view : Model -> Html AppMsg
view model =
    let
        line ( index, log ) =
            let
                ( bg, m ) =
                    case log of
                        LogOk m ->
                            ( "bg-green", m )

                        LogErr m ->
                            ( "bg-red", m )
            in
                div
                    [ class <| "bold white " ++ bg
                    , id <| "log_text_" ++ (toString index)
                    ]
                    [ button
                        [ class "btn"
                        , id <| "ack_log_" ++ (toString index)
                        , onClick <| AckLogMsg index
                        , title "acknowldge"
                        ]
                        [ i [ class "fa fa-times" ] [] ]
                    , text m
                    ]
    in
        div [] (List.map line <| Utils.enumerate model.logs)



-- METHODS


{-| log an error
-}
log : Model -> LogMsg -> Model
log model msg =
    let
        _ =
            case msg of
                LogOk m ->
                    Debug.log "OK: " m

                LogErr m ->
                    Debug.log "Err: " m
    in
        { model | logs = model.logs ++ [ msg ] }


invalidId : Model -> String -> Int -> Model
invalidId model desc id =
    log model <| LogErr <| desc ++ ": invalid id " ++ (toString id)
