## REQ-baz
implemented directly in source!

Not a partof anything...


## REQ-foo
```yaml art
partof: REQ-purpose
```
foo needs to do the foo thing


## REQ-lib
```yaml art
partof: REQ-purpose
```
Lib is definitely a library


## REQ-purpose
The purpose of this project is is to test a basic
project... that's it!


## SPC-build
```yaml art
partof: REQ-purpose
```
This has a build file.

Unit tests:
- [[.tst-unit]]


## TST-build
```yaml art
partof: REQ-purpose
```
direct link to REQ-purpose

- [[.no]]