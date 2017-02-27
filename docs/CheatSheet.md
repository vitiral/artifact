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
- **initialize repo**: `art init`
- **list/filter artifacts**: `art ls`
- **export a [static webpage][10]** `art export html`
- **check for errors**: `art check`
- **format design docs:** `art fmt`
- **get help**: `art [subcommand] -h`

[10]: https://vitiral.github.io/artifact-example/#artifacts/req-purpose

## Artifact Types
Artifact tracks "artifacts", which are objects which have a name, some text and
can be linked to other artifacts and to source code.

There are four types of artifact:
- `REQ`: requirement, *why* your application exists
- `RSK`: risk of a requirement, what you are concerned about
- `SPC`: specification of a requirement or higher-level spec. *How* you will
  build your program
- `TST`: details of what to test for a RSK or SPC

## Artifact Format
```
[REQ-name]
partof = REQ-other
text = '''
This is the description of the requirement
'''
```

- name looks like: `[REQ-name]`
- link them like: `partof = "REQ-[name, other, nested-[more, link]]"`
    - note: same as `partof = "REQ-name, REQ-other, REQ-nested-more,
      REQ-nested-link"`
- `SPC-name` is automatically partof `REQ-name` (if "name" is the same)
- `TST-name` is automatically partof `SPC-name` (if "name" is the same)
- `RSK` is not automatically linked to other types.
- `SPC-name-foo` is automatically partof `SPC-name` (same prefix) and `SPC-name`
  will be created if it doesn't exist.

## Settings
After `art init` settings are in: `.art/settings.toml`

Settings:
- `artifact_paths`: paths to design doc folders
- `code_paths`: paths to source code to link
- `exclude_code_paths`: paths of directories to exclude

## Linking to source
Writing `#SPC-name` anywhere in any valid utf-8 file will mark `SPC-name` as done.

Example:
```
// This is a comment about a function
// #SPC-name
fn get_name(raw: &str) -> String {
    return process_name(raw);
}
```
