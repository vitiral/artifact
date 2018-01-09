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
- **start the interactive tutorial**: `art tutorial`
- **get help**: `art [subcommand] -h`
- **initialize repo**: `art init`
- **list/filter artifacts**: `art ls`
- **check for errors**: `art check`
- **format artifacts:** `art fmt`
- **export a [static webpage][10]** `art export html --path-url $GITHUB_URL/blob/$(git rev-parse HEAD)/{path}#L{line}`
- **open an editable web-ui**: `art serve`
    - Has interactive help pages. Look for the `i` info symbols.

[10]: https://vitiral.github.io/artifact-example/#artifacts/req-1

## Artifact Types
Artifact tracks "artifacts", which are design documentation objects which have 
a name, some text and can be linked to other artifacts and to source code.

There are three types of artifacts:
- `REQ`: requirement, *why* your application exists
- `SPC`: specification of a requirement or higher-level spec. *How* you will
  build your program
- `TST`: details of what to test for a SPC

## Artifact Format
Artifact uses toml files to specify the design documents (artifacts).
The format looks like:
```toml
[REQ-name]
partof = 'REQ-other'
text = '''
This is the description of the requirement.

You can do soft-links to other artifacts:
- [[REQ-something]]: The web-ui will have a link to REQ-something
  and `art check` will make sure it exists.
'''
done = '''
If given, this will FORCE the artifact to be done.

Typically it is recommended to link to souce instead
'''
```

- name looks like: `[REQ-name]`
- link them like: `partof = ['REQ-name', 'REQ-other']`
    - shortcut exists for power users `partof = 'REQ-[name, other]'`. This
      can also be used as list items. `art fmt` will always convert it to the 
      long version. 
- `SPC-name` is automatically partof `REQ-name` (because "name" is the same)
- `TST-name` is automatically partof `SPC-name` (because "name" is the same)
- `SPC-name-foo` is automatically partof `SPC-name` (same prefix)
- `text` is a string that gets rendered as markdown (`.md`) via the web-ui.
  You should use it to define the artifact's purpose. 
- `done` is an arbitrary string that adds a 100% completed and tested sub-part
  (if it has no other sub-parts it will be 100% completed and tested). The
  artifact cannot be implemented in code if `done` is set.

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
```rust
// Documentation about the create_name function
//
// Implements #SPC-name
fn create_name(raw: &str) -> String {
    return process_name(raw);
}
```
