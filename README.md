# Artifact: design documentation for everybody

<img width="250" align="right" alt="artifact logo"
 src="https://github.com/vitiral/artifact/blob/master/docs/logo/logo.png?raw=true">

<a href="https://www.youtube.com/watch?v=kMzxKVkKLlE">
  <img width="250" align="right" alt="Introducing Artifact"
   src="docs/data/artifact-thumb.png">
</a>

- **[Installation Guide](docs/Installation.md)**
- **[Quick Start Guide](docs/QuickStart.md)**
- **[Cheat Sheet](docs/CheatSheet.md)**
- **[FAQ](docs/FAQ.md)**
- **[Simple Quality Book][1]**
- **[Rendered Design Documents][2]**
- **[Issue Tracker][6]**

Artifact is the simple, linkable and trackable design documentation tool for
everybody. It allows anyone to write and link their design documents both to
each other and to source code, making it easy to know how complete their
project is. Documents are revision controllable, can be edited in the browser
and have a full suite of command line tools for searching, displaying,
checking, exporting and formatting them.

<a href="https://twitter.com/b0rk/status/833419052194357248">
  <img align="right" src="docs/data/attribution/b0rk-design-documents.jpg-large"
    alt="b0rk scenes from writing design docs"
  >
</a>

Writing detailed design documents is one of the core pillars of quality software
development. Design documents are how you capture the requirements (purpose) of
your project and link them to your specifications (how you will build it). They
let you get your ideas on paper before writing code, and help you have fewer
painful refactors. They create a reference for developers and curious users of
how and why your project was developed a certain way, and make it easier to
refactor your project when that becomes necessary.

Even though design documents are critical to the quality of software, there
are very few tools for writing them and integrating them into the larger context
of a project. Artifact aims to fill the major gap in quality best practices by
making writing good design documents *useful* to the average developer.

First of all, artifact makes it easy to write design documents in *text files*
and link them by just specifying their `partof` attribute. This allows
developers to put their design documents under revision control, review them
using regular code review tools and use all the normal text processing tools
(vim, grep, sed, etc) to view, edit and refactor them. Artifact also provides
some command line tools of its own.

Secondly, design documents can be linked to source-code through a language
agnostic syntax (`#ART-name` anywhere in the source code). Once linked, anyone
reading the comment can easily look up the relevant design documents. In the
same way, anyone looking at the Web UI can see exactly where a specification or
test is implemented in code. Furthermore, if the name of a design doc changes,
`art check` will tell you where your dangling references are. Never again will
you have to be scared of refactoring your design documents because your
references in code will be out of date.

Finally, artifact exports a beautiful rendered view of your design documents
onto sites like github-sites ([example][2]) and you can edit in your browser
using `art serve`. This completes the self documenting nature and allows
anyone (even non-developers!) to view and edit the design documents of their
project.

In this way, artifact aims to unify other quality best practices while also
make writing design documents more fun and useful in your day to day
development efforts.

**Jump into artifact with the the [youtube commercial][4] and the
[Quick Start Guide](docs/QuickStart.md).**

## Support The Project
You can support the project by :star: staring it on github and
:green_heart: sharing it with your friends, coworkers and social media. You
can also support it directly [on patreon][5], vote for it in the
[2017 Hackaday Prize][7] and by leaving direct [feedback](docs/Feedback.md).

All funds collected through any link above  will **only** go towards hiring
student developers as interns to work on open source features within artifact.
**No money from these sources will go to myself (Garrett Berg) or to the
development of proprietary software.**

By supporting artifact, you are supporting open source tools for quality
software development and also internship oportunities for students passionate
about open source. Thank you!

## Stability
Artifact is 1.0 software with a strong commitment to backwards compatibility.
The 1.0 release is the "open source" release. Artifact is ready for projects of
any size to use it as their defacto design documentation tool, but the external
tooling may still be lacking for enterprise customers.

The 2.0 release will focus on stabilizing the library for external tooling.
This will position artifact for integration with industry tools such as JIRA
and external regression test tracking software. I am currently seeking
enterprise support, please consider [supporting this project on patreon][5].

The following are stable APIs that should always remain backwards compatible:
- Artifact `.toml` files. Features may be *added*, but should not be removed.
  This includes:
  - Artifact types: `REQ`, `SPC`, `TST`.
  - Artifact fields: `partof`, `text`, `done`.
  - Text format: markdown by default with `[[ART-name]]` links.
- Artifact relationships: explicit partof, auto linking by name, etc.
- Artifact completeness calculated from its relationships.
- Source code links: `#ART-name` anywhere in the source code marks
  the artifact as done.
- `.art/settings.toml` file
- The command line interface, including:
  - The name of the commands (`tutorial`, `init`, `ls`, `check`, `fmt`, etc).
  - Existing flags for each command.
  - The functional checks that are completed (i.e. dangling artifact names)
  - `art serve` default port of 5373 on localhost

The following should remain relatively stable but may have minor tweaks
before 2.0:
- The output of commands, including:
  - The format of the output messages. I.e. you should not rely on `art ls`
    having a specific output format.
  - The format of artifacts as performed by `art fmt`.
- The json format returned by `art ls --json` and through the json-rpc
  server from `art serve`.
- Anything not mentioned in the first section. If you are unsure, please
  open a ticket.

The following are expected to change a lot before 2.0:
- The web ui. Hopefully the changes will be an almost uniform improvement for
  everybody.
- The `art serve` http interface including its API methods and data format.
- The code and expected functions/types/etc of the library itself should be
  considered highly unstable. Future work will involve breaking it into smaller
  crates that are more stable.
- Logging messages evoked with `art -v`.
- Lots of other things that I can't think of. If you are unsure or concerned,
  open a ticket.

Artifact will continue to use a continuous-release cycle with extensive unit
and integration tests. There will also be a beta release channel for new and
experimental features. If you find a bug please [open a ticket][6].

[![Build Status](https://travis-ci.org/vitiral/artifact.svg?branch=master)](https://travis-ci.org/vitiral/artifact)

## Licensing

### Goals
The intent of the artifact licensing is that:
- The artifact application remains open source under a copy-left license
  but can be linked and built upon in any way (LGPLv3+)
- Anything generated by artifact can be licensed any way the user wishes
  to, including the built static html pages.
- Any documents, tutorials or specifications for artifact (except the code
  and logo) remain public domain and can be used for any purpose at all.

### Specifics
All documentation and tutorials for the artifact application are released under
the CC0 Creative Commons Public Domain License with the intent that you should
feel free to copy, paste and modify any of the designs, guides examples or
exported data for any purpose (including commercial) without the need of
attribution. You can read more about CC0 here:
https://creativecommons.org/publicdomain/

The CC0 license applies to:
- All project [docs](docs)
- The [Artifact Design Documents](http://vitiral.github.io/artifact/#artifacts/REQ-1)
    (also located in `design/`)
- The Artifact Documentation (located in `docs/`) except the logo in `docs/logo`
- The [Artifact Wiki](https://github.com/vitiral/artifact/wiki)
- Any file or data created by any artifact command, including:
    - documents created by `art init`
    - documents created by `art tutorial`
    - compiled html/css/json files created by `art export`

The artifact logo (named Tula) is licensed under Creative Commons
Attribution-ShareAlike (`CC BY-SA`) and can be used by the artifact project for
any purpose without needing additional attribution. The artifact logo is located
in `docs/logo` and was originally created by
[packapotatoes](https://github.com/packapotatoes).

The artifact source code (located in `src/` and `web-ui/src`) are licensed under
the LGPLv3+, except for files which say otherwise in their header or folders
containing a different `LICENSE` file. See [LICENSE.txt](LICENSE.txt) for more
information.

[1]: https://vitiral.gitbooks.io/simple-quality/content/
[2]: http://vitiral.github.io/artifact/#artifacts/REQ-1
[3]: https://github.com/vitiral/artifact/wiki/Exporting-Html
[4]: https://www.youtube.com/watch?v=kMzxKVkKLlE
[5]: https://www.patreon.com/user?u=7618979
[6]: http://github.com/vitiral/artifact/issues
[7]: https://hackaday.io/project/27132-artifact
