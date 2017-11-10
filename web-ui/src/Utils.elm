module Utils exposing (..)


isJust : Maybe m -> Bool
isJust v =
    case v of
        Just _ ->
            True

        Nothing ->
            False


isOk : Result e r -> Bool
isOk result =
    case result of
        Ok _ ->
            True

        Err _ ->
            False


assertOr : Bool -> a -> String -> a
assertOr test or_value msg =
    if test then
        or_value
    else
        Debug.crash msg


{-| convenience function for creating a list with indexes apparent
-}
enumerate : List a -> List ( Int, a )
enumerate list =
    List.indexedMap (,) list


{-| set the index of a list to a value

Note: this is NOT a safe function... nor is it fast...

-}
setIndexUnsafe : Int -> a -> List a -> List a
setIndexUnsafe index value list =
    let
        setter i v =
            if i == index then
                value
            else
                v
    in
        assertOr
            (index < List.length list)
            (List.indexedMap setter list)
            ("index " ++ (toString index) ++ " out of bounds")


strReplace : String -> String -> String -> String
strReplace pat replace_with original =
    String.split pat original
        |> String.join replace_with


unwrap : String -> Maybe a -> a
unwrap msg maybe =
    case maybe of
        Just v ->
            v

        Nothing ->
            Debug.crash ("Unwrap crashed: " ++ msg)

{-| find the index of a member

Crashes if the member doesn't exist or if there are more than one index

-}
memberIndexUnsafe : a -> List a -> Int
memberIndexUnsafe value list =
    let
        filter ( i, v ) =
            if value == v then
                Just i
            else
                Nothing

        indexes =
            List.filterMap filter (enumerate list)
    in
        if (List.length indexes) > 1 then
            Debug.crash "more than one member"
        else
            case List.head indexes of
                Just i ->
                    i

                Nothing ->
                    Debug.crash "member doesn't exist"


{-| Get the index of an array. Will panic if index doesn't exist.
This is horribly slow but elm is terrible at this stuff.
-}
getIndexUnsafe : Int -> List a -> a
getIndexUnsafe index list =
    let
        end =
            List.drop index list

        value =
            case List.head end of
                Just v ->
                    v

                Nothing ->
                    Debug.crash ("index " ++ (toString index) ++ " out of bounds")

    in
        value


{-| remove the value at index and return it and the
altered list
-}
popIndexUnsafe : Int -> List a -> ( a, List a )
popIndexUnsafe index list =
    let
        start =
            List.take index list

        end =
            List.drop index list

        value =
            case List.head end of
                Just v ->
                    v

                Nothing ->
                    Debug.crash ("index " ++ (toString index) ++ " out of bounds")

        end2 =
            List.drop 1 end
    in
        ( value, start ++ end2 )
