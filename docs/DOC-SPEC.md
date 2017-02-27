# Artifact Document Specification

This document outlines the specification for the Artifact data format.
All specifications in this document are released as Creative Commons CC0
public domain. You can read more about this license here:
https://creativecommons.org/publicdomain/

## Document Type

Artifact documents adhere to a subset of the [toml][1] format and are
documents of the form:

```
[ART-baz]
partof = "ART-baa"
text = '''
multi-line
description
'''

[ART-foo-bar]
partof = "ART-baz"
text = '''
multi-line
description
'''
```

## Artifact Types

Instead of `ART` as defined in Document Type, the user must select from
4 artifact types:
- `REQ`: specifying a requirement. `REQ` can be `partof` only
    `REQ`, `SPC` and `RSK`
- `SPC`: specifying a design specification. `SPC` can
    `partof` only `SPC` and `TST`
- `RSK`: specifying a risk with a `REQ`. `RSK` can
    `partof` only `RSK` and `TST`
- `TST`: specifying a test of a `SPC` or `RSK`. `TST` can
    `partof` only `TST`

## Automatic Creation

The following will be automatically created:
- parents: if `REQ-foo-bar` is specified but `REQ-foo`
    does not exist then it will be created. Parents
    of all artifacts are guaranteed to exist.

## Automatic Links

The following will be automatically linked:
- parents: `REQ-foo` will automatically be a `partof`
    `REQ-foo-bar`
- common-prefix for `REQ -> SPC -> TST` links
    - `REQ-foo` will automatically be a `partof` `SPC-foo`
        *if `REQ-foo` exists*
    - `SPC-foo` will automatically be a `partof` `TST-foo`
        *if `SPC-foo` exists*

[1]: https://github.com/toml-lang/toml
