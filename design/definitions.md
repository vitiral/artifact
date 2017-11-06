# SPC-0
done: by definition

partof: REQ-1
###
Definitions and process for the artifact project

## Assertions
Assertions **will** be used throughout the artifacts to mean:
- shall: the statement must be implemented and it's
    implementation verified.
- will: statement of fact, not subject to verification.
    i.e. "The X system will have timing as defined in ICD 1234"
- should: goals, non-mandatory provisions. Statements using "should"
    **should** be verified if possible, but verification is not mandatory if
    not possible. Is a statement of intent.

## License
All documentation for artifact including the Artifact Document Specification
and these design documents are both released under the CC0 Creative Commons
Public Domain License. You can read more about CC0 here:
https://creativecommons.org/publicdomain/

The artifact library and Web UI (located in `src/` and `web-ui/src`) are
licensed under the LGPLv3+, except for files which say otherwise in their
header or folders containing a different LICENSE.txt. See LICENSE.txt for more
information.

## Risks
Risks are to be written with three sets of terms in mind:
- likelyhood
- impact
- product placement

likelyhood has three categores:
 1. low
 2. medium
 3. high

impact has five categories:
 1. sand
 2. pebble
 3. rock
 4. boulder
 5. avalanche

product placement has three categores:
 1. cosmetic
 3. necessary
 5. critical

The value of these three categoires will be multiplied to
determine the weight to assign to the risk.

> sand may seem small, but if you have enough sand in your
> gears, you aren't going anywhere.
>
> You definitely need to watch out for boulders and prevent
> avalanches whenever possible

## Document Language
Possible choices:
 - json: decent but too general for the purpose (nesting is unnecessary)
     also, no way to line-comment making it almost useless for a tracking
     tool. Also, not readable enough.
 - ini/toml: both are decent formats, prefer toml as it is a little bit
     simpler
 - yaml: too complex for what is needed, not as readable as ini. Very
     enjoyable general purpose format.
 - html/xml/etc: aboslutely not. Needlessly complex.


The choice is TOML for writing artifact files.

The reasons are:
 - features covers all needs for artifact
 - easy to learn
 - easy to read
 - easy to write in
 - feature set well-limited to this application (not too many
     features or way to do the same thing)

## Programming Language
artifact's backend and cli app **will** be written entirely in the rust 
programming language for the purposes of:
- cross compilation: rust can be compiled on any platform
- safety: it is impossible to segfault in rust
- speed: rust is as fast as C++
- static checking: rust is one of the most powerful static type checking
    languages in existence, making it easier to refactor code
- scale-out: single threaded code can easily be made highly concurrent
- fun: rust is a fun language to write in.

The web-ui will be written in elm for many of the same reasons.

# TST-0
done: by definition
###
