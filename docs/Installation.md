This is the installation guide. For more information see the [[User Guide]]

artifact is compiled for linux, mac and windows. You can find releases on the
**[github release page](https://github.com/vitiral/artifact/releases)**.

For Linux and Mac, simply download and unpack the tarball with
`tar -zxvf RELEASE.tar.gz`. Then put it somewhere in your
[PATH](http://unix.stackexchange.com/questions/26047/how-to-correctly-add-a-path-to-path)

### Windows
> Note: windows guide is incomplete. Need to add how to add it to your "PATH"

For Windows, simply download the zip file (\*windows-gnu.zip for windows10),
unzip it and run `./artifact.exe` via git-bash.

### Arch Linux
In addition to the installation methods above, Artifact is maintained as a
package on the Arch AUR by [@rubdos][4]:

https://aur.archlinux.org/packages/artifact/

## Installing with [cargo](https://github.com/rust-lang/cargo)

Install rust with [rustup](https://github.com/rust-lang-nursery/rustup.rs) and
type `cargo install artifact-app` (upgrade with `-f`)

Note that as of version 0.6.4 this will not include the web server (but does have static
html export). That will be fixed in the next release.

## Installing the Server

**Instead of using the server most will want to use [`art export html`][2]**

The `server` command was removed in the run-up to the 1.0 release so that
development on it could continue without worry of backwards compatibility.
The server is currently only read-only, you are better off using
[static html][2] in almost all cases.

To install the server, you must clone the [server branch][3] and use Cargo to
build it.

```
cargo build --release --features server
```

Then copy `target/release/art` somewhere onto your `PATH`

[1]: https://github.com/vitiral/artifact
[2]: https://github.com/vitiral/artifact/blob/master/docs/ExportingHtml.md
[3]: https://github.com/vitiral/artifact/tree/server
[4]: https://github.com/rubdos
