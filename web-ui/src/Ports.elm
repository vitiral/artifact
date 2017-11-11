port module Ports exposing (..)

{-| Render the (text, part)
-}

-- GRAPHVIZ PORTS


port renderText : ( String, String ) -> Cmd msg


{-| Receive the rendered (text, part)
-}
port textRendered : (( String, String ) -> msg) -> Sub msg
