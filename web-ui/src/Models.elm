module Models exposing (..)

import Messages exposing (Route)
import Artifacts.Models exposing (Artifact, Artifacts, initialArtifacts)

type alias Settings =
  { readonly: Bool
  }

type alias Model =
  { artifacts: Artifacts
  , route: Route
  , errors: Errors
  , settings: Settings
  , addr: String
  }

type alias Errors =
  { descs: List String
  }

initialModel : String -> Route -> Model
initialModel addr route =
  { artifacts = initialArtifacts
  , route = route
  , errors = initialErrors
  , settings = initialSettings
  , addr = addr
  }

initialSettings : Settings
initialSettings =
    -- FIXME: change to True
  { readonly = False
  }

initialErrors : Errors
initialErrors = 
    { descs = []
    }


appendError : Model -> String -> Model
appendError model err =
  let
    _ = Debug.log err

    errors = model.errors

    newErrors = { errors | descs = List.append errors.descs [err] }
  in 
    { model | errors = newErrors }
