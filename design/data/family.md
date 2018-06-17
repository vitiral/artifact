# SPC-family
partof: REQ-data
###
An artifact (name) has the following "family" defined:

```dot
digraph G {
    subgraph cluster_allowed {
        label=<<b>allowed partof</b>>;
        REQ -> SPC -> TST;
        REQ -> TST;

        REQ -> REQ;
        SPC -> SPC;
        TST -> TST;
    }

    subgraph cluster_relationship {
        label=<<b>auto family</b>>;
        "REQ-root"
            -> {"REQ-root-child" [color=blue]}
            [label="is parent of"; color=blue; fontcolor=blue];
        "REQ-root" -> "SPC-root" [label="is auto-partof"];
        "SPC-root"
            -> {"SPC-root-child" [color=blue]}
            [label="is parent of"; color=blue; fontcolor=blue];
        "SPC-root" -> "TST-root" [label="is auto-partof"];
    }
}
}
```

## Allowed Partof
The first graph shows what relationships are "allowed". It specifies that:
- `REQ` can be `partof` any type
- `SPC` can be `partof` `SPC` and `TST`
- `TST` can only be `partof` itself.

In essense:
- You can always create "subtypes", i.e. a more specific requirement
- You can create a specification that is "partof" a requirement. This makes
  sense as you want to define your specifications based on your requirements.
- You can create a test that is "partof" a specification OR a requirement.
  For example, white box testing will be based on a specification whereas
  blackbox ("requirements based") testing will be based on a requirement.

## Lints
Lints are required to make sure the above is upheld

- [[.lint_partof_exists]]: Make sure any partof references actually exist.
- [[.lint_types]]: Make sure that `partof` links are only made between valid types.

## [[.auto]]: Auto Relationships
The second graph shows the "automatic relationships" of nodes to their
parents.

- A node is automatically a `partof` both its parent and it's auto-partof.
- Artifacts that have only one element are "root" (i.e. REQ-root, REQ-foo, SPC-foo)
- Any artifact that is *not* root has a single parent, which it will automatically
  be a "partof". That parent **must** be defined by the user or it is a hard error
- SPC and TST artifacts have auto-partof elements of the higher-order type (see
  [[SPC-name]]. This element is **not required** to exist, but if it does
  they will be linked automatically.

A node can always be partof another node of the same type. In addition, the following type links are allowed

```dot

```


# SPC-read-family
The method of determining family is fairly straightforward, as is
detailed in the graph below:

```dot
digraph G {
    [[.parent]] -> { "if" [label="if elem-len > 1"; shape=diamond] };
    "if" -> "return None" [label = "no"];
    "if" -> {"clone raw string" [
        shape=box;
    ]} -> {"pop last element and create new Name" [
        shape=box;
    ]} -> "return new name";

    [[.auto_partof]] -> { if_req [label="type is REQ"; shape=diamond] };
    if_req -> "return None" [label="yes"];
    if_req -> {"get higher-order type" [
        shape=box;
    ]} -> {"clone raw name" [
        shape=box;
    ]} -> {"swap type with higher-order one" [
        shape=box;
    ]} -> "return new Name";
}
```

# [[.auto]]
Once family is created and the artifacts are loaded, the artifacts have
to be automatically linked to their parent+auto_partof. This is easy
to determine given the artifacts that exist.

Note: make sure to ONLY link to artifacts that exists!

# [[.deauto]]
In order to reserialize the artifacts, their "auto" partof has to be unlinked