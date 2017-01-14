[![Build Status](https://travis-ci.org/vitiral/rst.svg?branch=master)](https://travis-ci.org/vitiral/rst)
# rst: the requirements tracking tool made for developers

**rst** is a [requirements tracking](https://en.m.wikipedia.org/wiki/Software_requirements_specification)
tool made for developers. It is an acronym for "Requirements, Specifications and Tests".

**rst** is pronounced like "wrist"

Check out the **[Wiki](https://github.com/vitiral/rst/wiki)** for the User Guide and other
information.

Use the **[Issue Tracker](https://github.com/vitiral/rst/issues)** if you find any bugs
or have major questions.

## Installation
See the [User Guide](https://github.com/vitiral/rst/wiki/User-Guide) on the wiki.

### Beta Notice
**rst** is still in Beta and is not 100% feature complete. The API for the cmdline and
text format is expected to be stable, but the author reserves the right to change anything
that needs to be changed if it improves usability.

**Future improvements include:**
 - Additional command line tools
     - export: export artifacts to json, csv, html and other formats
 - Test Tracking: REST API with DB backend for tracking test execution
     - plus cmdline utility and webui for viewing test execution.
         (rst currently only supports tracking implementaiton, not execution)
 - The Web UI is currently read-only. It will be able to edit soon.

## Licensing
The rst file format (the format of the toml files, artifact name, etc) is
licensed under the CC0 (Creative Commons Public Domain) License. Any person can
use the format for any reason without the need for even attribution (attribution
is appreciated though!)

The rst library and web-ui are licensed under the LGPLv3+, except for files
which say otherwise in their header. See LICENSE.txt for more information.

