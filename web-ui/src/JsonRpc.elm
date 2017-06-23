module JsonRpc exposing (..)

{-| module for defining JSON RPC related models
-}


type alias RpcError =
    { code : Int
    , message :
        String

    --, data: Nothing
    }


formatJsonRpcError : RpcError -> String
formatJsonRpcError error =
    "JSON-RPC Error: " ++ error.message
