## Why is it named artifact?
Artifact is simply named after what it does: it is a way to write and track
your [artifacts](https://en.wikipedia.org/wiki/Artifact_(software_development))

### Why is (extended) markdown the default language?
Because it is human/readable and writeable. Adding the metadata block was also
not difficult and fit within the syntax.

### An artifact is "implemented" in code but not 100% done?
All artifacts are only as done as their parts + implementation/done.

If you have:
```toml
[SPC-1]
[SPC-1-a]
[SPC-1-b]
```

And then the code:
```python
def hello():
    """partof: #SPC-1"""
```

`SPC-1` will only be 1/3 "done" since it still has two incomplete parts.

This also applies to the "done" field.
