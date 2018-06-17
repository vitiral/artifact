# SPC-structs
partof: REQ-data
###
This requirement details the high level `struct`s and `enum`s that must be
exported by this module, as well as their features.

In many cases this would be a *specification*, but since this is a library,
the exported structs and their characteristics practically ARE the
requirement.

It's critical that the valid types are defined at a high level, since
they determine how everything works together.

### Exported Types
These are the types that make up the exported "product" of this library. The
major type is the **Artifact** and its associated **Name**.

TODO: the original graph broke with the update to newer graphiz. Rewrite with the format below.

```dot
digraph G {
    node [shape=record];
    Animal [
        label =
        "{Animal
        \l|+ name : string
        \l+ age : int
        \l|+ die() : void
        \l}"
    ]
}
```

### Raw Data Types
These types define the "raw data" format of artifact and are only used
for de/serializing.

#### [[.artifact_raw]]: ArtifactRaw: (stored with key of `Name`)
- done: `Option[String]`
- partof: `Option[HashSet[Name]]`
- text: `Option[TextRaw]`

#### [[.text_raw]]: TextRaw: just a newtype with some serialization guarantees
to make it prettier and ensure serialization is possible between
all of the supported formats.

### Intermediate (`Im`) Data Types
Intermediate "stripped down" forms of the artifact. These types are used for:
- linting after reading the project
- inteacting with the CRUD interface.

#### [[.artifact_op]]: ArtifactOp:
- `Create(ArtifactIm)`: create an artifact, it must not already exist.
- `Update(HashIm, ArtifactIm)`: update the artifact with the specified hash.
- `Delete(HashIm)`: delete the artifact with the specifed hash.

This is the "operation" command used by [[SPC-modify]] for modifying artifacts.
`Read` is ommitted as it is covered by [[SPC-read]].

#### [[.artifact_im]]: ArtifactIm:
- name: `Name`
- file: `PathAbs`
- partof: `Set<Name>` (auto-partofs are stripped)
- done: `Option<String>`
- text: `String`

The `ArtifactIm` is used to create a unique 128 bit hash of the artifacts and
for specifying *what* should be updated when an update is requested.

This is also the primary type used when linting.

#### HashIm:
This is simply a 128 bit SipHash created by the [`siphasher` crate][1].

[1]: https://doc.servo.org/siphasher/sip128/struct.Hash128.html


## Type Details

**Artifact**: ([[.artifact]]) the artifact is the primary exported type. It contains:
  - `name`: the unique identifier of the artifact.
  - `file`: the file where the artifact is defined.
  - `partof` and `parts`: automatic and user-defined relationship to other
    artifacts where `B in A.partof` means that B is a "parent" of A.
    More details are in [[SPC-family]].
  - `completed`: the `spc` and `tst` completion ratios, detailing how much of
    the artifact's specification and test design has been implemented.
  - `text`: the user defined text in the markdown format.
  - `impl_`: how the artifact is implemented (if it is implemented at all). Can
    be `Done(str)`, `Code(ImplCode)` or `NotIMpl`.
  - `subnames`: a list of subnames defined in `text` using `{{.subname}}`
    except `[[]]` instead of `[[]]`. These can be linked in code to complete
    the artifact.
  - `orig_hash`: the original hash of the `ArtifactIm` this was created from.

**Name**:
  - name is of the form `ART-name` where ART is one of {`REQ`, `SPC` or `TST`}
  - more details are in [[SPC-name]].

**Impl**:
  - Defines how the artifact is implemented.
  - `Done`: can be "defined as done" through the `done` field in the
    `ArtifactRaw`.
  - `Code`: can be implemented in code, where source code just puts `#ART-name`
    anywhere to mark an artifact as implemented.