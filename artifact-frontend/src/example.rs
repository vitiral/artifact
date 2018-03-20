
pub const YAML: &str = r###"
paths:
    code_paths: []
    exclude_code_paths: []

    artifact_paths:
    - /fake/design
    exclude_artifact_paths: []

code_impls: {}

artifacts:
    REQ-purpose:
        name: REQ-purpose
        file: /fake/design/purpose.md
        partof: []
        parts: []
        completed: {spc: 0.0, tst: 0.0}
        text: This text was updated
        impl_: null
        subnames: []
    REQ-single:
        name: REQ-single
        file: /fake/design/purpose.md
        partof: []
        parts: []
        completed: {spc: 0.0, tst: 0.0}
        text: |-  # note `|-` => strip newline at the end
            This is a single line of text
        impl_: null
        subnames: []
    REQ-completed:
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
        impl_: null
        subnames:
            - .one
            - .two
    SPC-completed:
        name: SPC-completed
        file: /fake/design/purpose.md
        partof:
            - REQ-completed
        parts: []
        completed: {spc: 1.0, tst: 1.0}
        text: |-
            Just marked as done
        impl_: "this is done"
        subnames: []
    SPC-wut:
        name: SPC-wut
        file: /fake/design/purpose.md
        partof:
            - REQ-completed
        parts: []
        completed: {spc: 1.0, tst: 1.0}
        text: |-
            Just marked as done
        impl_: "this is done"
        subnames: []
"###;
