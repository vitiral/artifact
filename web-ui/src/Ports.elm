port module Ports exposing (..)

import Artifacts.Models exposing (ArtifactId)


-- GRAPHVIZ PORTS

-- Render the artifacts
port renderArtifacts : List (ArtifactId, String) -> Cmd msg


-- Receive the rendered artifacts
port artifactsRendered : (List (ArtifactId, String) -> msg) -> Sub msg
