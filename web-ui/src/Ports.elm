port module Ports exposing (..)

import Artifacts.Models exposing (ArtifactId)


-- GRAPHVIZ PORTS
-- Render the artifacts


port renderText : String -> Cmd msg



-- Receive the rendered artifacts


port textRendered : (String -> msg) -> Sub msg
