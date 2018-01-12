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

# TST-name
partof: TST-fuzz
###
The `Name` type is fairly low level with no dependencies, so interop testing
is not necessary.

- [[.sanity_valid]]: assert that names are valid in the general use case as well
  as edge cases (one element, more than one element, etc)
- [[.sanity_invalid]]: assert that names are invalid for all edge cases
  (extra `--`, `REQ` by itself, `REQ-`, `REQ-a-`, etc).
- [[.sanity_serde]]: do basic check that serde works with names.
- [[.fuzz]]: fuzz definitions shall be applied to be used both here and
  externally.
- [[.fuzz_name_roundtrip]]: check that any two names are equal if their keys
  are equal.
