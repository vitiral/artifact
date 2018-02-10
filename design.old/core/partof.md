# SPC-partof
The partof attribute shall follow the following spec for specifying multiple artifacts
- `[]` will denote a set of artifacts related to the prevoius characters
- `,` will denote separate artifact groups

# Example:
Both of these are the same set of artifacts:
- `bar-d[1, 2, 3]`
- `bar-d1, bar-d2, bar-d3`

Note: spaces ` ` are always ignored

## invalid inputs
- the `,` character is **only** permitted inside of brackets
- after a closing bracket `]`, **only** the `,` or closing `]` character is permitted
    - i.e. `REQ-[foo, bar]-baz` is invalid
- it is an error for a `LOC` artifact to be in `partof` (use `loc`)

# valid inputs
Some interesting valid use cases are:
- `REQ-[foo, bar-[1, 2, 3, 6], baz]` evaluates to `REQ-foo, REQ-bar-[1, 2, 3, 6], REQ-baz`
    which evaluates to `REQ-foo, REQ-bar-1, REQ-bar-2, REQ-bar-3, REQ-bar-6, REQ-baz`

The reasons for these rules are:
- without the availability to do ranges or re-use a prefix, it can be very annoying to
    list the requirements in an easy to understand way
- however, the use cases for wanting to do something like [a,b]-[1,2,3] are poor.
    The names and purpose of sub-categories should rarely (if ever) be
    connected, and trying to maintain structures like that would likely cause
    the developer more harm than good.
- the parser is much easier to implement, and it is easier to see "under the hood"
    at what the parser is actually doing if it only uses simple recursion rather than
    having to go backwards in the parsing.

## Loading partof
The Artifact names **shall** be processed from plaintext and stored in a structure

multi-name strings **shall** be parsed with a simple linear function pushes
onto a `String` datatype.

# parts
1. read one character at a time
2. validates the character
3. recurses if a `[]` block is being entered

It will **not** validate that the artifact names are correct -- that is the
responsibility of `::from_str`.
