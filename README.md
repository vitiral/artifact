[![Build Status](https://travis-ci.org/vitiral/artifact.svg?branch=master)](https://travis-ci.org/vitiral/artifact)

# artifact: the design doc tool made for developers

- [**How To Install**](https://github.com/vitiral/artifact/wiki/User-Guide)
- [**Additional Information**](https://github.com/vitiral/artifact/wiki)
- [**Reporting Issues**](https://github.com/vitiral/artifact/issues)

### Beta Notice
**artifact** is still in Beta and is not 100% feature complete. The API for the cmdline and
text format is expected to be stable, but the author reserves the right to change anything
that needs to be changed if it improves usability.

### Tasks to Reach 1.0
- []: create document spec for "artifact document" and make CC0
- [x]: UpdateArtifacts API call
- []: CreateArtifacts API call
- []: DeleteArtifacts API call
- [x]: Web UI View functionality
- []: Web UI search/filter functionality
- []: Web UI markdown rendering
- []: Web UI edit functionality
- []: Web UI create/delete functionality
- []: extensive selenium testing of Web UI
- []: only allow settings in `.art/settings.toml`
- []: *remove* `globals` setting (text-variables)
- []: fix any stop-ship bugs

#### Future (post 1.0) Improvements
- []: export to json
- []: export to markdown
- []: JSON-RPC API for Test Execution Tracking with DB backend
- []: cmdline utility for viewing tracked tests
- []: Web UI for viewing tracked tests

## Licensing
The artifact file format (the format of the toml files, artifact name, etc) is
licensed under the CC0 (Creative Commons Public Domain) License. Any person can
use the format for any reason without the need for even attribution (attribution
is appreciated though!)

The artifact library and Web UI are licensed under the LGPLv3+, except for files
which say otherwise in their header. See LICENSE.txt for more information.

