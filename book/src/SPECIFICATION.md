# Artifact Document Specification

This document outlines the specification for the Artifact data format.
All specifications in this document are released as Creative Commons CC0
public domain. You can read more about this license here:
https://creativecommons.org/publicdomain/

## Document Type

Artifact documents can be specified in multiple formats.

### TOML Format
The TOML format adheres to a subset of the [TOML][1] format and are documents
of the form:

```toml
[ART-baz]
partof = "ART-baa"
text = '''
multi-line
description
'''

[ART-foo-bar]
partof = [
    "ART-baz",
]
text = '''
multi-line
description
'''
```

Where `partof` can be either a single string or a list of strings.

### Markdown Format
The markdown format uses extended Commonmark (**TODO: link**) format.
It is of the form:

```
# REQ-foo
<optional yaml section for metadata>
###
<markdown text>
```

Where the yaml section is completely optional (the `###` can be skipped if it
doesn't need it.

## Artifact Types

Instead of `ART` as defined in Document Type, the user must select from
3 artifact types:
- `REQ`: specifying a requirement. `REQ` can only have `REQ` in its
  `partof` field.
- `SPC`: specifying a design specification. `SPC` can only have
  `REQ` or `SPC` in its `partof` field.
- `TST`: specifying a test of a `SPC`. `TST` can have any of
  `REQ`, `SPC` or `TST` in its `partof` field.

## Automatic Links

The following will be automatically linked:
- parents: `REQ-foo` will automatically be a `partof`
    `REQ-foo-bar`
- common-prefix for `REQ -> SPC -> TST` links
    - `REQ-foo` will automatically be a `partof` `SPC-foo`
        *if `REQ-foo` exists*
    - `SPC-foo` will automatically be a `partof` `TST-foo`
        *if `SPC-foo` exists*

## Linking an artifact in source code
Artifacts can be linked in source code, which "completes" their `spc%`.

The way to link to an artifact is to place `#ART-name` anywhere in the source
code file.

## Sub Artifacts (subart)
A sub artifact is defined by placing `[[.subart]]` anywhere in the `text` field
of an artifact.

Subarts can be linked in source code as well by placing `#ART-name.subart`
anywhere in the source code file.

A special subart is `[[.tst-subart]]` which will contribute to both `tst%` and
`spc%`.


[1]: https://github.com/toml-lang/toml
