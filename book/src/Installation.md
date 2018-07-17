This is the installation guide. For more information see the
[project home page](https://github.com/vitiral/artifact)

## Typical Installation
artifact is compiled for linux, mac and windows. You can find releases on the
**[github release page](https://github.com/vitiral/artifact/releases)**.

For Linux and Mac, simply download and unpack the tarball with
`tar -zxvf <release_name>.tar.gz`. Then put it somewhere in your [PATH][10]

[10]: http://unix.stackexchange.com/questions/26047/how-to-correctly-add-a-path-to-path

You can then update to the newest version with `art update`

### Windows

The recommended method of installation for windows is to use the **scoop**
package manager.

First, [install scoop](http://scoop.sh/) from a powershell terminal:
```
iex (new-object net.webclient).downloadstring('https://get.scoop.sh')
```

Then install artifact:
```
scoop install artifact
```

### Arch Linux
In addition to the installation methods above, Artifact is maintained as a
package on the Arch AUR by [@rubdos][4]:

https://aur.archlinux.org/packages/artifact/

## Building From Source
Simply execute the following:
```bash
git clone https://github.com/vitiral/artifact
cd artifact
cargo build --release
```

> Note: you may need `cargo-web` installed as well.

Do a full suite of tests with:
```bash
cargo test
```

## Installing with [cargo](https://github.com/rust-lang/cargo)

Install rust with [rustup](https://github.com/rust-lang-nursery/rustup.rs) and
type `cargo install artifact-app`

Note this may never be feature complete and is not the recommended method of
installation.

[1]: https://github.com/vitiral/artifact
[2]: https://github.com/vitiral/artifact/blob/master/docs/ExportingHtml.md
[3]: https://github.com/vitiral/artifact/tree/server
[4]: https://github.com/rubdos
