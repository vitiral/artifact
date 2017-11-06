# SPC-artifact
High level specification for the Artifact type including:
- struct definition
- loading

Artifacts shall be specified by users in a toml file loaded into an internal 
struct using serde.

The artifact struct shall contain all fields that are loaded from the user
and processed by the application. This includes:
 - path: file where this artifact is defined
 - partof: list of artifacts this artifact is a partof
 - parts: calculated list of artifacts that art parts of this one
 - text: user input text describing this artifact
 - loc: location in source code where this artifact is implemented
 - completed: percent completed
 - tested: percent tested

name is an attribute because it is used as the key for the Artifacts 
type.
