Here is an ultra rough overview of the tool for those already familiar with it
or who can get up to speed quickly.

An example repository you can play around with is here:
[https://github.com/vitiral/artifact-example](https://github.com/vitiral/artifact-example)

## Useful Background
- [Installation Guide](./Installation.html)
- [FAQ](./FAQ.html): frequently asked questions about artifact
- [Best Practices](./BestPractices.html): artifact best practices
- [Vocabulary](./Vocabulary.html): useful vocabulary for writing design documents

[1]: https://vitiral.gitbooks.io/simple-quality/content/vocabulary.html

## Useful commands
- `art help`: get help
- `art [subcommand] -h`: get help on a subcommand.
- `art init`: initialize repo
- `art serve`: open an editable Web UI.
- `art ls`: list/filter artifacts
- `art check`: check for errors
- `art fmt`: format artifacts
- `art export html $DEST`: export a [static webpage](examples/part2/index.html)

[10]: https://vitiral.github.io/artifact-example

## Artifact Types
Artifact tracks "artifacts", which are design documentation objects which have
a name, some text and can be linked to other artifacts and to source code.

There are three types of artifacts:
- `REQ`: Requirement, _why_ your application exists. Also used for high level
  architecture/goals. Typically these relate to the user in some way.
- `SPC`: Specification of how a requirement will be implemented. _How_ you will
  build your program. Typically these are to document for developers how or why
  something is implemented a certain way (from a larger architectural point of
  view).
- `TST`: Test. Details of what to test for a SPC or REQ.

## Artifact Format
Artifacts can be speicifed in three formats: markdown (`.md`), TOML (`.toml`)
or YaML (`.yaml`).

### Markdown Format
Artifact uses markdown (by default) to specify the design documents (artifacts). The
complete format looks like:


    # REQ-name
    partof:
    - REQ-other
    - REQ-foo
    done: This artifact is "defined as done".
    ###
    The description of the artifact goes here.

    You can do soft-links to other artifacts:
    - [[REQ-something]]: The web-ui will have a link to REQ-something and `art
      check` will make sure it exists.


- name looks like: `# REQ-name`
- The `partof` metadata field is how you link artifacts of any name.
    - power user feature: `partof: REQ-[name, other]` is the same as being
      partof both `REQ-name` and `REQ-other`. This can also be used as list
      items. `art fmt` will always convert it to the long
      version.
- The `done` metadata is an arbitrary string that adds a 100% completed and tested sub-part
  (if it has no other sub-parts it will be 100% completed and tested). The
  artifact cannot be implemented in code if `done` is set.
- `SPC-name` is automatically partof `REQ-name` (because "name" is the same)
- `TST-name` is automatically partof `SPC-name` (because "name" is the same)
- `SPC-name-foo` is automatically partof `SPC-name` (same prefix)

Note that if no metadata is specified you can simply write:
```
# REQ-name
The description of the artifact goes here.
```

## TOML Format

Toml used to be the default format

    [REQ-name]
    partof = [
        'REQ-other',
        'REQ-foo',
    ]
    done = 'This artifact is "defined as done"'
    text = """
    The description of the artifact goes here.
    """


## Settings
After running `art init`, your settings will be in: `.art/settings.toml`

Settings:
- `artifact_paths`: paths to directories/files containing artifacts (in `.toml`
  files)
- `exclude_artifact_paths`: paths of directories/files to exclude from
  `artifact_paths`.
- `code_paths`: paths of source code containing `#ART-name` references.
- `exclude_code_paths`: paths of directories/files to exclude from `code_paths`

## Implementing artifacts and subarts
Writing `#SPC-name` in any valid utf-8 file (read: source code file) that is in
a `code_paths` path will mark the artifact `SPC-name` as done.

You can also specify subarts (pieces of an artifact that should be implemented
in code) by putting `[[.subart]]` anywhere in an artifact's `text` field. These
can be linked in code like so: `#ART-name.subart`.

In addition, artifact supports specifying unit tests using a `[[.tst-name]]`
subart. These subarts contribute to both `spc%` and `tst%`.

Example Artifact:
```markdown
# SPC-name
This has [[.subart]] subart.

Also [[.tst-name]] unit test.
```

Example Code Implementation:
```python
#!/usr/bin/python
def create_name(raw):
    """Documentation about the create_name function

    Implements #SPC-name

    Also implements #SPC-name.subart
    """
    return process_name(raw);


def test_create_name():
   """#SPC-name.tst-name"""
   assert create_name("foo") == "FOO"
```
