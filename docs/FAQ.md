## Why is it named artifact?
Artifact is simply named after what it does: it is a way to write and track your
[artifacts](https://en.wikipedia.org/wiki/Artifact_(software_development)

## Why is the `partof` attribute a string (not a list)?
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
