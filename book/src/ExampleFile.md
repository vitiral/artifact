# REQ-learn
Welcome to the artifact tutorial! This file is written just like artifact
markdown files are. Artifact files can be written in a range of formats, the
currently supported ones being markdown, toml and yaml.

An artifact file is simply a set of artifacts, each one written like so:
```
# REQ-NAME
<regular markdown section here>
```

Artifacts can be a requirement (REQ), design-specification (SPC)
or test (TST)

The artifact you are reading now is a requirement, therefore it begins with
"REQ".


# REQ-markdown
partof:
- REQ-learn
###

Artifact files like this one are written in a slightly extended markdown format
you can read more about it here: http://commonmark.org/help/tutorial/

The "extended" part is that artifact treats the following syntax as special:
```
# ART-name
<optional yaml section here>
###
<regular markdown section here>
```

Where `ART` is one of `REQ`, `SPC`, `TST` and `<optional yaml here>` is a few
items like `partof` and `done` fields. We will get to those later.


# SPC-learn
partof:
- REQ-markdown
###

Anything starting with SPC is a design specification. Comparing them:

Requirements (REQ) should be used for
- detailing what you want your application to do
- what the architecture of your applicaiton should be

Specifications (SPC) should be used for
- how you intend to write your application (lower level details).

There are also tests (TST) which we will learn about later.


# SPC-partof
partof:
- REQ-learn
###

Artifact uses the names of artifacts to automatically link them and track
progress. This makes it easy for the user to intuitively link together
requirements with their specification and reduces boilerplate.

[[SPC-learn]] is automatically a "partof" [[REQ-learn]] because the names after
the type are the same ("-learn").

You can also explicitly link artifacts like so:
```
# ART-name
partof:
- ART-other
- <additional partof>
###
<regular markdown section here>
```

So far we have:
```
REQ-learn
    ^
    |
    +-- REQ-markdown
    |       ^
    |       |
    +<----- SPC-learn
    |
    \-- REQ-partof <-- SPC-partof
```

Here is a helpful graph of valid partof relations between artifacts:
```
  REQ <-- SPC <-- TST
```

Therefore:
- A REQ can be partof a REQ only
- A SPC an be partof a REQ or SPC
- A TST can be partof a REQ, SPC or TST

# SPC-valid

There are only a few rules for defining artifacts:
 - Case is ignored for all names.
 - Names cannot overlap, even in different files.
 - All names must start with either REQ, SPC or TST.


# TST-definition
TST artifacts are used to document test design and are the only way that an
artifact can be considered "tested" (besides the `done` field).

Artifact makes it easy to track the "progress" of your application because `art
ls` (and the web-ui) gives you easy to easy to read completion and tested
percentages for all your artifacts based on which ones are implemented in
source code (more on that later).
