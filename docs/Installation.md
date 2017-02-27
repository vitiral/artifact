This is the installation guide. For more information see the [[User Guide]]

artifact is compiled for linux, mac and windows. You can find releases on the
**[github release page](https://github.com/vitiral/artifact/releases)**.

For Linux and Mac, simply download and unpack the tarball with
`tar -zxvf RELEASE.tar.gz`. Then put it somewhere in your
[PATH](http://unix.stackexchange.com/questions/26047/how-to-correctly-add-a-path-to-path)

> Note: windows guide is incomplete. Need to add how to add it to your "PATH"

For Windows, simply download the zip file (\*windows-gnu.zip for windows10),
unzip it and run `./artifact.exe` via git-bash.

## Installing with [cargo](https://github.com/rust-lang/cargo)

Install rust with [rustup](https://github.com/rust-lang-nursery/rustup.rs) and
type `cargo install artifact-app` (upgrade with `-f`)

Note that as of version 0.6.1 this will not include the web server (but does have static
html export). That will be fixed in the next release.
