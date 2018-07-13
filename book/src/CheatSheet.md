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
-  `art tutorial`: start the interactive tutorial
- `art [subcommand] -h`: get help
- `art init`: initialize repo
- `art ls`: list/filter artifacts
- `art check`: check for errors
- `art fmt`: format artifacts
- `art export html --path-url $GITHUB_URL/blob/$(git rev-parse HEAD)/{path}#L{line}`: export a [static webpage][10]
- `art serve`: open an editable web-ui
    - Has interactive help pages. Look for the `i` info symbols.

[10]: https://vitiral.github.io/artifact-example/#artifacts/req-1

## Artifact Types
Artifact tracks "artifacts", which are design documentation objects which have
a name, some text and can be linked to other artifacts and to source code.

There are three types of artifacts:
- `REQ`: requirement, *why* your application exists. Also used for high level
  architecture/goals as they relate to the user.
- `SPC`: specification of how a requirement will be implemented. *How* you will
  build your program
- `TST`: details of what to test for a SPC or REQ.

## Artifact Format
Artifact uses markdown to specify the design documents (artifacts). The
complete format looks like:
```
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
```

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

## Settings
After `art init` settings are in: `.art/settings.toml`

Settings:
- `artifact_paths`: paths to directories/files containing artifacts (in `.toml`
  files)
- `exclude_artifact_paths`: paths of directories/files to exclude from
  `artifact_paths`.
- `code_paths`: paths of source code containing `#ART-name` references.
- `exclude_code_paths`: paths of directories/files to exclude from `code_paths`

## Linking to source
Writing `#SPC-name` in any valid utf-8 file (read: source code file) that is in
a `code_paths` path will mark the artifact `SPC-name` as done.

Example:
```python
#!/usr/bin/python
def create_name(raw):
    """Documentation about the create_name function

    Implements #SPC-name
    """
    return process_name(raw);
```
