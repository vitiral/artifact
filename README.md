[![Build Status](https://travis-ci.org/vitiral/artifact.svg?branch=master)](https://travis-ci.org/vitiral/artifact)

**artifact** is a design doc tool made for developers. It makes it easy to write and link your design docs and then
track them using any revision control tool that you love. **artifact** creates
[rendered static html pages](http://vitiral.github.io/artifact/) so that others can easily view how your project is
designed.

- [Design Docs](http://vitiral.github.io/artifact/) ([How to do this](https://github.com/vitiral/artifact/wiki/Exporting-Html))
- [**How To Install**](https://github.com/vitiral/artifact/wiki/User-Guide)
- [**Additional Information**](https://github.com/vitiral/artifact/wiki)
- [**Reporting Issues**](https://github.com/vitiral/artifact/issues)

### Beta Notice
**artifact** is still in Beta and is not 100% feature complete. The API for the cmdline and
text format is expected to be stable, but the author reserves the right to change anything
that needs to be changed if it improves usability.

### Tasks to Reach 1.0
- [x] design doc review and cleanse
- [ ] create document spec for "artifact document" and make CC0
- [x] included static web-file generation
- [x] Web UI View functionality
- [ ] extensive selenium testing of Web UI
- [x] *remove* `globals` setting (text-variables)
- [ ] only allow settings in `.art/settings.toml`
- [ ] add setting `additional_repos`
- [ ] fix any stop-ship bugs

#### Future (post 1.0) Improvements
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

