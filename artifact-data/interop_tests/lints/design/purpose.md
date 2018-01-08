# REQ-exists
partof:
- REQ-dne       # dne
- SPC-exists    # invalid type
- TST-exists    # invalid type
###

# SPC-exists
partof:
- TST-exists    # invalid type
###
- [[.sub]]: has a sub, horray

# TST-exists
done: "this is done baby"
###
but... also has subparts
[[.foo]]

## Invalid references
[[REQ-dne]]   [[REQ-dne.sub]]

[[REQ-exists.dne]]

## Valid references
[[REQ-exists]]

[[SPC-exists]]      [[SPC-exists.sub]]
