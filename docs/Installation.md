This is the installation guide. For more information see the [[User Guide]]

## Typical Installation
artifact is compiled for linux, mac and windows. You can find releases on the
**[github release page](https://github.com/vitiral/artifact/releases)**.

For Linux and Mac, simply download and unpack the tarball with
`tar -zxvf RELEASE.tar.gz`. Then put it somewhere in your [PATH][10]

[10]: http://unix.stackexchange.com/questions/26047/how-to-correctly-add-a-path-to-path

### Windows
> Note: windows guide is incomplete. Need to add how to add it to your "PATH"

For Windows, simply download the zip file (\*windows-gnu.zip for windows10),
unzip it and run `./artifact.exe` via git-bash.

### Arch Linux
In addition to the installation methods above, Artifact is maintained as a
package on the Arch AUR by [@rubdos][4]:

https://aur.archlinux.org/packages/artifact/

## Building From Source
Simply execute the following:
```
git clone https://github.com/vitiral/artifact
cd artifact
source env  # this will take a while
cargo build --features server --release
```

Do a full suite of tests with:
```
just test-all
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
