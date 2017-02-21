[![Build Status](https://travis-ci.org/vitiral/artifact.svg?branch=master)](https://travis-ci.org/vitiral/artifact)

Artifact is a design doc tool made for developers. It allows anyone to
easily write and link their design docs both to each other and to source code,
making it easy to track how complete their project is.
Documents are revision controllable, can be rendered as a static
[web page][2] and have a full suite of command line tools for searching,
formatting and displaying them.

The current release is targeted towards open source developers. Future releases
aim to support industry by allowing editing of artifacts via the Web UI, as
well as tracking and graphing of test execution across their product's versions.

- [**Design Docs**][2]: also see how [you can do this][1]
- [**How To Install**][3]
- [**Additional Information**][4]
- [**Reporting Issues**][5]

[1]: https://github.com/vitiral/artifact/wiki/Exporting-Html
[2]: http://vitiral.github.io/artifact/
[3]: https://github.com/vitiral/artifact/wiki/User-Guide
[4]: https://github.com/vitiral/artifact/wiki
[5]: https://github.com/vitiral/artifact/issues

### Beta Notice
Artifact is nearly feature complete. The following will change prior to 1.0:

### Tasks to Reach 1.0
- [x] design doc review and cleanse
- [ ] create document spec for "artifact document" and make CC0
- [x] included static web-file generation
- [x] Web UI View functionality
- [x] *remove* `globals` setting (text-variables)
- [ ] only allow settings in `.art/settings.toml`
- [ ] add setting `additional_repos`
- [ ] fix any stop-ship bugs

#### Future (post 1.0) Improvements
- [ ] extensive selenium testing of Web UI
- [ ] Web UI search/filter functionality
- [ ] Web UI markdown rendering
- [ ] Web UI edit functionality
- [ ] Web UI create/delete functionality
- [x] UpdateArtifacts API call
- [ ] CreateArtifacts API call
- [ ] DeleteArtifacts API call
- [ ] cmdline settings (in `settings.toml::cmd`)
- [ ] export to json
- [ ] export to markdown
- [ ] JSON-RPC API for Test Execution Tracking with DB backend
- [ ] cmdline utility for viewing tracked tests
- [ ] Web UI for viewing tracked tests

## Licensing
The artifact file format (the format of the toml files, artifact name, etc) is
licensed under the CC0 (Creative Commons Public Domain) License. Any person can
use the format for any reason without the need for even attribution (attribution
is appreciated though!)

The artifact library and Web UI are licensed under the LGPLv3+, except for files
which say otherwise in their header. See LICENSE.txt for more information.

