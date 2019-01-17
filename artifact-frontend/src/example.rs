/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
pub const YAML: &str = r###"
settings:
    base: "/fake"
    code_paths: []
    exclude_code_paths: []

    artifact_paths:
    - /fake/design
    exclude_artifact_paths: []

    settings_path: "/fake/.art/settings.toml"

code_impls: {}

artifacts:
    REQ-purpose:
        id: "gQ7cdQ7bvyIoaUTEUsxMsg"
        name: REQ-purpose
        file: /fake/design/purpose.md
        partof: []
        parts: []
        completed: {spc: 0.0, tst: 0.0}
        text: This text was updated
        impl_:
            type: NotImpl
            value: null
        subnames: []
    REQ-single:
        id: "gp7cdQ7bvyIoaUTEUsxMsg"
        name: REQ-single
        file: /fake/design/purpose.md
        partof: []
        parts: []
        completed: {spc: 0.0, tst: 0.0}
        text: |-  # note `|-` => strip newline at the end
            This is a single line of text
        impl_:
            type: NotImpl
            value: null
        subnames: []
    REQ-completed:
        id: "gp9cdQ7bvyIoaUTEUsxMsg"
        name: REQ-completed
        file: /fake/design/purpose.md
        partof: []
        parts:
            - SPC-completed
            - SPC-wut
        completed: {spc: 0.25, tst: 1.0}
        text: |
            Basic demonstration of completeness

            Has some subnames:
            - [[.one]]
            - [[.two]]
        impl_:
            type: NotImpl
            value: null
        subnames:
            - .one
            - .two
    SPC-completed:
        id: "gp9ckQ7bvyIoaUTEUsxMsg"
        name: SPC-completed
        file: /fake/design/purpose.md
        partof:
            - REQ-completed
        parts: []
        completed: {spc: 1.0, tst: 1.0}
        text: |-
            Just marked as done
        impl_:
            type: Done
            value: "this is done"
        subnames: []
    SPC-wut:
        id: "gp9ckQ7bvyzoaUTEUsxMsg"
        name: SPC-wut
        file: /fake/design/purpose.md
        partof:
            - REQ-completed
        parts: []
        completed: {spc: 1.0, tst: 1.0}
        text: |-
            Just marked as done
        impl_:
            type: Done
            value: "this is done"
        subnames: []
"###;
