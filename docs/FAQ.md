## Why is it named artifact?
Artifact is simply named after what it does: it is a way to write and track your
[artifacts](https://en.wikipedia.org/wiki/Artifact_(software_development)

### Why is the `partof` attribute a string (not a list)?
The partof attribute is much more powerful than a simple list. You can do:
```
[SPC-name]
partof = "REQ-[name, other, nested-[more, link]]"
```
Which is equivalent to:
```
[SPC-name]
partof = "REQ-name, REQ-other, REQ-nested-more, REQ-nested-link"
```

### An artifact is "implemented" in code but not 100% done?
All artifacts are only as done as their parts + implementation/done.

If you have:
```
[SPC-1]
[SPC-1-a]
[SPC-1-b]
```

And then the code:
```
def hello():
    """partof: #SPC-1"""
```

`SPC-1` will only be 1/3 "done" since it still has two incomplete parts.

This also applies to the "done" field.
