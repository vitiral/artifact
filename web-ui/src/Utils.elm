module Utils exposing (..)


isJust : Maybe m -> Bool
isJust v =
    case v of
        Just _ ->
            True

        Nothing ->
            False
