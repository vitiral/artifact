# SPC-name
partof: REQ-data
###
The following attributes must be definable by the user:
- `name`: the artifact name must be given in the form `ART-name`, where `ART`
  is used to determine the type (see below).
- `done`: if any string is given, the artifact is "defined as done", meaning it
  is 100% complete for both implementation and test.
- `partof`: a list (or compressed syntax) of artifact names which this artifact
  is a "partof". Valid and automatic links are defined in [[SPC-family]].
- `text`: the description of the artifact which can contain "soft links" to
  other artifacts as well as to code implementations.

## [[.type]]:  Artifact Type
The type of an artifact is simply its prefix, which must be one of:
- `REQ`: requirement
- `SPC`: design specification
- `TST`: test specification

The order of precedence is:
- `REQ` is "higher order" than `SPC` or `TST`
- `SPC` is "higher order" than `TST`

```dot
digraph G {
    graph [rankdir=LR; splines=ortho]
    REQ -> SPC -> TST
}
```

See [[SPC-family]] for how these are related.

## [[.attrs]]: Attributes/Getters

The `Name` type shall be the exported "key" of artifacts.  Internally it is
reference counted, externally it exposes itself with the following methods:
- `Name.ty`: get the name's type
- `Name.from_str(s)`: create or automatically load the name.
- `Name.as_str()`: get the string representation of the name. This must always
  be the same string as the user gave.
- `Name.key_str()`: get the name's "key" representation

Internally the name is an atomically reference counted pointer (`Arc`), meaning
that cloning it is extremely cheap.