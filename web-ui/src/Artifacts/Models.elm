module Artifacts.Models exposing (..)

import Set
import Dict
import Regex
import JsonRpc exposing (RpcError)


-- CONSTANTS


artifactValidRaw : String
artifactValidRaw =
    "(REQ|SPC|TST)(-[A-Z0-9_-]*[A-Z0-9_])?"


artifactValidPat : Regex.Regex
artifactValidPat =
    Regex.regex <| "^" ++ artifactValidRaw ++ "$"


artifactsUrl : String
artifactsUrl =
    "#artifacts"


newId : String
newId =
    "new-artifact"


-- TYPES


type alias Project =
    { artifacts : Artifacts
    , paths : ProjectPaths
    }

type alias ProjectPaths =
    { base: String
    , code_paths : Set.Set String
    , exclude_code_paths : Set.Set String
    , artifact_paths : Set.Set String
    , exclude_artifact_paths : Set.Set String
    }

type alias ProjectData =
    { artifacts : List Artifact
    , paths : ProjectPaths
    , code_impls: Dict.Dict String ImplCode
    , checked : String
    }

type alias Completed =
    { spc: Float
    , tst: Float
    }

{-| representation of an Artifact object
-}
type alias Artifact =
    { id : ArtifactId
    , name : Name
    , file : String
    , text : String
    , partof : List Name
    , parts : List Name
    , completed : Completed
    , impl : Impl
    , subnames : List String
    , edited: Maybe EditableArtifact
    }

type Impl
    = Done String
    | Code ImplCode
    | NotImpl

type alias ImplCode =
    { primary: Maybe Loc
    , secondary: Dict.Dict String Loc
    }

type alias FullLocs =
    { root : Maybe Loc
    , sublocs : Dict.Dict String Loc
    }


type alias Loc =
    { file : String
    , line : Int
    }


type alias Name =
    { raw : String
    , value : String
    , ty : Type
    }


{-| pretty much only used when updating artifacts
-}
type alias ArtifactId =
    String


{-| the type of the artifact name
-}
type Type
    = Req
    | Spc
    | Tst


{-| the standard lookup method for artifacts
-}
type alias NameKey =
    String


{-| How artifacts are stored
-}
type alias Artifacts =
    Dict.Dict ArtifactId Artifact


{-| Editable part of an artifact
-}
type alias EditableArtifact =
    { name : String
    , file : String
    , partof : List Name
    , done : Maybe String
    , text : String
    , original_id: ArtifactId
    }


type alias ProjectResponse =
    { result : Maybe ProjectData
    , error : Maybe RpcError
    }


type EditOption
    = ChangeChoice Artifact EditableArtifact
    | CreateChoice EditableArtifact


type ViewOption
    = ReadChoice Artifact
    | EditChoice EditOption



-- INIT


initialArtifacts : Artifacts
initialArtifacts =
    Dict.empty


{-| get the real name from a raw name
return Err if name is invalid
-}
indexName : String -> Result String String
indexName name =
    let
        index =
            indexNameUnchecked name
    in
        if Regex.contains artifactValidPat index then
            Ok index
        else
            Err ("Invalid name: " ++ name)


{-| used for ONLY internal name handling when we
convert to/from the DOM.
-}
initNameUnsafe : String -> Name
initNameUnsafe raw =
    case initName raw of
        Ok n ->
            n

        Err e ->
            Debug.crash e


{-| initialize a name object and find it's type
-}
initName : String -> Result String Name
initName name =
    case indexName name of
        Ok value ->
            case getType value of
                Just ty ->
                    Ok
                        { raw = name
                        , value = value
                        , ty = ty
                        }

                Nothing ->
                    -- this should NEVER happen
                    -- (except for some internal usage)
                    Err "Critical: invalid artifact type"

        Err e ->
            Err e


{-| convert a list of artifacts to a dictionary by Name
-}
artifactsFromList : List Artifact -> Artifacts
artifactsFromList artifacts =
    let
        pairs =
            List.map (\a -> ( a.id, a )) artifacts
    in
        Dict.fromList pairs



-- METHODS


{-| returns the type of the artifact based
on it's name.

Notice: this function is unsafe for unknown strings!

-}
getType : NameKey -> Maybe Type
getType name =
    case String.left 3 name of
        "REQ" ->
            Just Req

        "SPC" ->
            Just Spc

        "TST" ->
            Just Tst

        _ ->
            Nothing


{-| returns whether name can be a partof `partof`

i.e. can name have `partof` in its "partof" field

-}
validPartof : Name -> Name -> Bool
validPartof name partof =
    if name == partof then
        False
    else
        case ( name.ty, partof.ty ) of
            -- req only partof req
            ( Req, Req ) ->
                True

            -- spc only partof req or spc
            ( Spc, Req ) ->
                True

            ( Spc, Spc ) ->
                True

            -- test only partof spc or tst
            ( Tst, Spc ) ->
                True

            ( Tst, Tst ) ->
                True

            -- all others false
            _ ->
                False


{-| return whether `partof` will automatically be put in the
partof field of an artifact named name
-}
autoPartof : Name -> Name -> Bool
autoPartof name partof =
    let
        name_sp =
            String.split "-" name.value

        partof_sp =
            String.split "-" partof.value

        name_len =
            List.length name_sp

        partof_len =
            List.length partof_sp
    in
        if partof_len == name_len - 1 && (List.take partof_len name_sp) == partof_sp then
            -- name is the prefix for `partof`
            True
        else if
            (validPartof name partof)
                && ((List.drop 1 name_sp) == (List.drop 1 partof_sp))
        then
            -- `partof` is valid partof with same postfix
            True
        else
            False


isRead : ViewOption -> Bool
isRead option =
    case option of
        ReadChoice _ ->
            True

        EditChoice _ ->
            False


{-| gets the edited variable of the artifact
or creates the default one
-}
getEditable : Artifact -> EditableArtifact
getEditable artifact =
    -- FIXME
    -- case artifact.edited of
    --     Just e ->
    --         e

    --     Nothing ->
            createEditable artifact


createEditable : Artifact -> EditableArtifact
createEditable artifact =
    { name = artifact.name.raw
    , file = artifact.file
    , text = artifact.text
    , partof = artifact.partof
    , done =
        case artifact.impl of
            Done s ->
                Just s

            _ ->
                Nothing
    , original_id = artifact.id
    }


editedDebug : EditableArtifact -> String
editedDebug e =
    [ "name = " ++ e.name
    , "file = " ++ e.file
    , "text = " ++ e.text
    , "partof = " ++ (String.join ", " <| List.map (\p -> p.raw) e.partof)
    , "done = " ++ (Maybe.withDefault "NULL" e.done)
    ]
        |> String.join " "


{-| the edited artifacts are equal if their non-automatic
partof is equal and everything except revision is equal.

Always returns false if either name is invalid

-}
editedEqual : EditableArtifact -> EditableArtifact -> Bool
editedEqual e1 e2 =
    case ( initName e1.name, initName e2.name ) of
        ( Ok n1, Ok n2 ) ->
            let
                nonAuto name partof =
                    List.filter (\p -> not <| autoPartof name p) partof
                        |> List.map (\p -> p.raw)

                sanitize =
                    (\e -> { e | partof = []})

                p1 =
                    Set.fromList <| nonAuto n1 e1.partof

                p2 =
                    Set.fromList <| nonAuto n2 e2.partof
            in
                (p1 == p2) && ((sanitize e1) == (sanitize e2))

        _ ->
            False


getEdited : EditOption -> EditableArtifact
getEdited option =
    case option of
        ChangeChoice _ e ->
            e

        CreateChoice e ->
            e


setEdited : EditOption -> EditableArtifact -> EditOption
setEdited option edited =
    case option of
        ChangeChoice a _ ->
            ChangeChoice a edited

        CreateChoice _ ->
            CreateChoice edited


getNameIndex : EditOption -> String
getNameIndex option =
    case option of
        ChangeChoice artifact _ ->
            artifact.name.value

        CreateChoice _ ->
            "CREATE"


getArtifactId : EditOption -> ArtifactId
getArtifactId option =
    case option of
        ChangeChoice artifact _ ->
            artifact.id

        CreateChoice _ ->
            "FIXME: fake"


artifactNameUrl : String -> String
artifactNameUrl name =
    "#artifacts/" ++ (String.toLower name)


{-| get the real name from a raw name
-}
indexNameUnchecked : String -> String
indexNameUnchecked name =
    String.toUpper name



-- VIEW Models -- TODO: move this somewhere else


{-| artifact attributes which can be displayed
or searched for
-}
type alias Columns =
    { parts : Bool
    , partof : Bool
    , text : Bool
    , file : Bool
    , loc : Bool
    }


initialColumns : Columns
initialColumns =
    { parts = True
    , partof = False
    , text = True
    , file = False
    , loc = False
    }


type alias Search =
    { pattern :
        String

    -- the pattern to search for
    , name : Bool
    , parts : Bool
    , partof : Bool
    , text : Bool
    }


initialSearch : Search
initialSearch =
    { pattern = ""
    , name = True
    , parts = False
    , partof = False
    , text = False
    }


type alias TextViewState =
    { rendered_edit :
        Bool

    -- display the rendered tab for edit view
    , rendered_read :
        Bool

    -- display the rendered tab for read view
    }


initialTextViewState : TextViewState
initialTextViewState =
    { rendered_edit = False
    , rendered_read = True
    }
