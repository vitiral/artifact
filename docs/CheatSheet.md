Here is an ultra rough overview of the tool for those already familiar with it:

An example repository you can play around with is here:
    https://github.com/vitiral/artifact-example

## Useful Background
- [Installation Guide](Installation.md)
- [FAQ](FAQ.md): frequently asked questions about artifact
- [Best Practices](BestPractices.md): artifact best practices
- [Vocabulary][1]: useful vocabulary for writing design documents
- [Simple Quality][2]: short book which is the primary tutorial for this tool
  while also teaching quality best practices

[1]: https://vitiral.gitbooks.io/simple-quality/content/vocabulary.html
[2]: https://www.gitbook.com/read/book/vitiral/simple-quality/

## Useful commands
- **get help**: `art [subcommand] -h`
- **initialize repo**: `art init`
- **list/filter artifacts**: `art ls`
- **check for errors**: `art check`
- **format artifacts:** `art fmt`
- **export a [static webpage][10]** `art export html --path-url $GITHUB_URL/blob/$(git rev-parse HEAD)/{path}#L{line}`
- **open an editable web-ui**: `art serve`

[10]: https://vitiral.github.io/artifact-example/#artifacts/req-1

## Artifact Types
Artifact tracks "artifacts", which are objects which have a name, some text and
can be linked to other artifacts and to source code.

There are three types of artifact:
- `REQ`: requirement, *why* your application exists
- `SPC`: specification of a requirement or higher-level spec. *How* you will
  build your program
- `TST`: details of what to test for a SPC

## Artifact Format
Artifact uses toml files to specify the design documents (artifacts).
The format looks like:
```
[REQ-name]
partof = REQ-other
text = '''
This is the description of the requirement
'''
done = '''
If given, this will FORCE the artifact to be done.

Typically it is recommended to link to souce instead
'''
```

- name looks like: `[REQ-name]`
- link them like: `partof = "REQ-[name, other, nested-[more, link]]"`
    - note: same as `partof = "REQ-name, REQ-other, REQ-nested-more,
      REQ-nested-link"`
- `SPC-name` is automatically partof `REQ-name` (because "name" is the same)
- `TST-name` is automatically partof `SPC-name` (because "name" is the same)
- `SPC-name-foo` is automatically partof `SPC-name` (same prefix)
- `done` is an arbitrary string that marks any artifact as 100% completed and
  tested.

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
Writing `#SPC-name` anywhere in any valid utf-8 file that is in a `code_paths` file
or directory will mark the artifact `SPC-name` as done.

Example:
```
// Documentation about the create_name function
// #SPC-name
fn create_name(raw: &str) -> String {
    return process_name(raw);
}
```
