error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        // no external error chains (yet)
    }

    foreign_links {
        // stdlib
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);

        // crates
        TomlDecode(::toml::DecodeError);
        StrFmt(::strfmt::FmtError);
    }

    errors {
        // Loading errors
        Load(desc: String) {
            description("Misc error while loading artifacts")
            display("Error loading: {}", desc)
        }
        TomlParse(locs: String) {
            description("Error while parsing TOML file")
            display("Error parsing TOML: {}", locs)
        }
        MissingTable {
            description("Must contain a single table")
        }
        InvalidName(desc: String) {
            description("invalid artifact name")
            display("invalid artifact name: \"{}\"", desc)
        }
        InvalidAttr(name: String, attr: String) {
            description("artifact has invalid attribute")
            display("Artifact {} has invalid attribute: {}", name, attr)
        }
        InvalidSettings(desc: String) {
            description("invalid settings")
            display("invalid settings: {}", desc)
        }
        InvalidArtifact(name: String, desc: String) {
            description("invalid artifact")
            display("artifact {} is invalid: {}", name, desc)
        }

        // Processing errors
        InvalidTextVariables {
            description("couldn't resolve some text variables")
        }
        InvalidPartof {
            description("Some artifacts have invalid partof attributes")
        }
        InvalidDone {
            description("Some artifacts have invalid partof attributes")
        }
        NameNotFound(desc: String) {
            description("searched for names were not found")
            display("the following artifacts do not exists: {}", desc)
        }
        LocNotFound {
            description("errors while finding implementation locations")
        }
        DoneTwice(desc: String) {
            description("the artifact is done and implemented in code")
            display("referenced in code and `done` is set: {}", desc)
        }
        InvalidUnicode(path: String) {
            description("we do not yet support non-unicode paths")
            display("invalid unicode in path: {}", path)
        }

        // Cmd errors
        CmdError(desc: String) {
            description("error while running a command")
            display("{}", desc)
        }

        // Misc errors
        PathNotFound(desc: String) {
            description("invalid path")
            display("Path does not exist: {}", desc)
        }
        NotEqual(desc: String) {
            description("values not equal")
            display("{}", desc)
        }
        Security(desc: String) {
            description("security vulnerability detected")
            display("security vulnerability: {}", desc)
        }
        Internal(desc: String) {
            description("internal error")
            display("internal error: {}", desc)
        }
        NothingDone {
            description("internal control flow")
        }
    }
}
