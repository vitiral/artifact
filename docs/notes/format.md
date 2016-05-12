
# General technologies used
- The formatting language that will be used is [toml](https://github.com/toml-lang/toml)
- the language the tool will be written in is rust


# Format

The format for requirements will follow this spec

## Categories
There will be X categories:
- REQ: software requriement
- RSK: Requirements Risks -- risks related to the software requirements
- TST: high level test design
- SPC: High Level Software Design Spec
- LOC: location of code implementation

all items **must** :
- have one of the above prefixes
- must be composed of characters in the set [\d\w-]
- case will be ignored
- spaces are allowed and will be ignored

There is a specialized language for listing multiple requirements
- `[]` denotes a set of requirements related to the prevoius characters
- `,` separates requirements
- `:` denotes a range of inclusive values (numerical or alphabetic)
- `*` denotes that all sub-requirements are met

### Example: 

Both of these are the same set of requirements
- `foo-[a:c], foo-d[1,2,3]`
- `foo-a, foo-b, foo-c, foo-d1, foo-d2, foo-d3`

multi-character ranges are understood:
- `[9:12]` == `[9,10,11,12]`
- `[z:ac]` == `[z,aa,ab,ac]`

## Links and Relation

```
Some basic rules
- links are always valid within the same category (i.e. REQ to REQ)
  - all items that have an identical substring surrounded by "-" are linked
    - i.e. "foo" and "foo-bar" are linked but "foz-bar" and "foz-baz" are not linked
    - these links do not need to be made explicitly (they are implicit)
- links are done between different categories and must be done as follows:
  - REQ -> nothing              # requirements cannot link (except to other REQ)
  - SPC -> REQ + LOC            # you can implement a design spec which fulfills requirement(s)
  - RSK -> REQ, SPC             # there can be a risk associated with a requirement or spec
  - TST -> REQ, SPC, RSK + LOC  # you can implement a test that tests a requirement, spec or risk

## Completeness

percent completeness is calculated using the "weight" field and through the following algorithm

In the example:
```
[REQ-foo]
links = "REQ-bar"

[REQ-bar]
```

- REQ-bar is not considered complete/tested until REQ-foo is complete/tested
- REQ-foo can be complete without REQ-bar being complete
- it is illegal to put `links = "REQ-foo"` in REQ-bar

Here is a graph of the relations (without LOC)
```
  REQ<----SPC<----RSK<+
   ^       ^       |  |
   |       |       |  |
   +---------------+  |
   |       |          |
   |       |          |
   -------TST---------+
```

## Implementation completeness of items
- A REQ is considered 100% implemented when all linked REQs and SPCs are implemented
- a SPC is considered 100% implemented when all linked SPCs are implemented

## Testing completeness of items
- a RSK is considered 100% tested when all linked TSTs are implemented
- a SPC is considered 100% tested when all linked TSTs are implemented AND
    all linked RSKs are tested
- a REQ is considered 100% tested when all linked TSTs are implemented AND
    all linked SPCs and RSKs are tested

## Document Layout

A typical document will look like:

```
# Requirements document
# "#" can be used for comments

# settings fields modify various things about all items
[settings]
prefix = "REQ-foo-"  # add a prefix to all keys in this file
reqs = "REQ-foh, REQ-foa"  # add additional linked REQs to all items in this file

# variables can be accessed through the "{}" notation
# [localvars] and [globalvars] are special headings that cannot be used elsewhere
# variables that are not in "" must be a float/int, variables used in "" must be a string
[localvars]  # these variables are available locally
codepath = "{src}/mylib"

# these are injected into the global scope. If there are overlaps there will be an error
# They can be retrieved through "{}"
[globalvars]
FOOBAR = foo_bar

[bar-definition]
doc = "bar means foo"
test = false  # don't include this requirement in test coverage

[bar]  # requirement REQ-foo-bar  (see prefix)
doc = "The foo **must** do bar"
active = true
weight = 2
reqs = "REQ-baz-[1,2]"  # associated requirements

[bar-1]  # requirement REQ-foo-bar-1  (see prefix)
doc = "bar **must** do 1"
active = false
weight = 2.5
```

