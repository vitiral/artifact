port module Ports exposing (..)

-- GRAPHVIZ PORTS


{-| Render the (text, part)
-}
port renderText : (String, String) -> Cmd msg


{-| Receive the rendered (text, part)
-}
port textRendered : ((String, String) -> msg) -> Sub msg
