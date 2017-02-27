- **[Quick Start Guide](docs/QuickStart.md)**
- **[Cheat Sheet](docs/CheatSheet.md)**
- **[FAQ](docs/FAQ.md)**
- **[Simple Quality][1]**: book which uses this tool to teach quality best
  practices.
- **[Design Documents][2]**: also see how [you can do this][3]

[1]: https://vitiral.gitbooks.io/simple-quality/content/
[2]: http://vitiral.github.io/artifact/#artifacts/REQ-1
[3]: https://github.com/vitiral/artifact/wiki/Exporting-Html

Artifact is a design doc tool made for developers. It allows anyone to
easily write and link their design docs both to each other and to source code,
making it easy to track how complete their project is.
Documents are revision controllable, can be rendered as a static
web page and have a full suite of command line tools for searching,
formatting and displaying them.

The current release is targeted towards open source developers. Future releases
aim to support industry by allowing editing of artifacts via the Web UI, as
well as tracking and graphing of test execution across their product's versions.

[![Build Status][build-status]][travis]
[build-status]: https://travis-ci.org/vitiral/artifact.svg?branch=master
[travis]: https://travis-ci.org/vitiral/artifact

### Pre-release notice

Artifact is now feature complete for 1.0. The 0.6 release has been released and
will have about a month long soak process while the tool is used by as many
projects as possible. Further changes before 1.0 are not expected, but may
still be necessary. After the soak, the plan is to cut the 1.0 release which
will disallow backwards incompatible changes.

#### Future Improvements
- [ ] web-ui settings (in `.art/web-ui.toml`)
- [ ] Extensive selenium testing of Web UI
- [x] UpdateArtifacts API call
- [ ] CreateArtifacts API call
- [ ] DeleteArtifacts API call
- [ ] Web UI search/filter functionality
- [ ] Web UI markdown rendering
- [ ] Web UI edit functionality
- [ ] Web UI create/delete functionality
- [ ] cmdline settings (in `.art/cmd.toml`)
- [ ] JSON-RPC API for Test Execution Tracking with DB backend
- [ ] cmdline utility for viewing tracked tests
- [ ] Web UI for viewing tracked tests

## Contributors

Please check out the [Contributor Guide][20]

[20]: https://github.com/vitiral/artifact/wiki/Contributor-Guide

## Licensing
All documentation and tutorials for the artifact application are released under
the CC0 Creative Commons Public Domain License with the intent that you should
feel free to copy, paste and modify any of the designs, guides or examples
for any purpose without the need of attribution. You can read more about CC0 here:
https://creativecommons.org/publicdomain/

The CC0 license applies to:
- The [Artifact Document Specification](docs/DOC-SPEC.md)
- The [Artifact Design Documents](http://vitiral.github.io/artifact/#artifacts/REQ-1)
    (also located in `design/`)
- The [Artifact Wiki](https://github.com/vitiral/artifact/wiki)
- Any documents created by `art init` (in `src/cmd/data`)
- Any documents created by `art tutorial` (n `src/cmd/data`)

The artifact library and Web UI (located in `src/` and `web-ui/src`) are licensed
under the LGPLv3+, except for files which say otherwise in their header or folders
containing a different LICENSE.txt. See LICENSE.txt for more information.
