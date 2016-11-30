module Models exposing (..)

import Messages exposing (Route)
import Artifacts.Models exposing (Artifact)

type alias Settings =
  { readonly: Bool
  }

type alias Model =
  { artifacts: List Artifact
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
  { artifacts = []
  , route = route
  , errors = initialErrors
  , settings = initialSettings
  , addr = addr
  }

initialSettings : Settings
initialSettings =
  { readonly = True
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
