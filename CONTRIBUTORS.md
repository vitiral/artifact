## Contributors
**[Chat on gitter](https://gitter.im/artifact-app/Lobby)**

To set up a build environment and run tests, simply run:

```bash
git clone git@github.com:vitiral/artifact.git && cd artifact
source env  # installs environment to `target/env`
just test-all
```

Note: `source env` will take a while as it (locally) installs
build/test/lint/fmt toolchains for rust, node.js and python from scratch. It
does not touch ANYTHING in your global environment.

A quick source code overview:
- `justfile` contains build/test/etc scripts
- design documents are in `design/`
- rust source code is in `src/`
- elm source code (html frontend) is in `web-ui/src/`
- selenium (end-to-end web) tests are in `web-ui/sel_tests/`
