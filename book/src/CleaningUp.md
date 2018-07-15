# Cleaning Up
There are two more commands that it is critical to know:
- `art check`: for checking for errors and warnings.
- `art fmt`: for formatting your arguments.

`art check` checks a whole range of things:
- All artifacts in `partof` exist.
- The soft references (i.e. `[[REQ-foo]]`) exist.
- Code references are valid and not duplicated.

`art fmt` standardizes the format of your artifacts and makes them easier to read.

> Note: `art fmt` is automatically run whenever you edit any artifacts via the
> Web UI.


## Documenting and Hosting your own project
To start documenting your own project, run `art init` in your project and
edit `.art/settings.toml` with the paths on where to find your
design docs and code.

Have your build system export your design documents as html for easy viewing.
See: [Exporting Html](./ExportingHtml.html)


## Artifact Advice
Here are a words when using artifact:

1. You should always write a good README and other documentation for your users
   -- design docs SHOULD be used for bringing developers of your project up
   to speed but they aren't the best format for general users.
2. Keep your design docs fairly high level -- don't try to design every detail
   using artifact. Using artifact does not mean that you shouldn't use code
   comments!
3. Use `art ls` and `art check` often, and fix those error messages!
4. Follow the [artifact best practices](./BestPractices.html).
5. Don't be afraid to refactor your design docs. It is actually easier than it
   might sound, as the tool will help you find broken links and incomplete
   items in real time. Not to mention that if you use revision control
   (you should), your artifacts can be tracked with your project -- no more
   having your documentation and your code be wildly out of sync!

This tutorial took you part of the way through developing a simple project
using artifact. Continue onto the next section or simply try using artifact for
one of your smaller personal projects and see the benefits that design
documents can give -- it's your choice!

Have some fun with the tool, try to break it. If you find bugs or have any
suggestions, please open a ticket at:
https://github.com/vitiral/artifact/issues

Good luck!
