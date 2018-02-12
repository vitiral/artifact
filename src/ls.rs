/* artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
//! #SPC-cli-ls
use std::io;

use dev_prelude::*;
use artifact_data::*;
use termstyle::{self, Color, El, Table, Text};
use termstyle::Color::*;

macro_rules! t { [$t:expr] => {{
    Text::new($t.into())
}}}

#[derive(Debug, StructOpt)]
#[structopt(name = "ls", about = "List and filter artifacts")]
#[cfg_attr(rustfmt, rustfmt_skip)]
// #SPC-cli-ls.args
pub struct Ls {
    #[structopt(long = "verbose", short = "v", default_value="0")]
    /// Pass many times for more log output.
    pub verbosity: u64,

    #[structopt(long="work-dir")]
    /// Use a different working directory [default: $CWD]
    pub work_dir: Option<String>,

    #[structopt(name="PATTERN")]
    /// Regular expression to search for artifact names.")]
    pub pattern: Option<String>,

    #[structopt(short="f", long="fields", value_name="FIELDS",
      default_value="name",
      help="\
Specify fields to search for the regular expression PATTERN.

Valid FIELDS are:
- N/name: search the \"name\" field (artifact name)
- F/file: search the \"file\" field (see -F)
- P/parts: search the \"parts\" field (see -P)
- O/partof: search the \"partof\" field (see -O)
- T/text: search the \"text\" field (see -T)

Fields can be listed by all caps, or comma-separated lowercase.

Both of these commands will list only artifacts with \"foobar\" in the name
or text fields of all artifacts.

    art ls foobar -p NT
    art ls foobar -p name,text

Regular expressions use the rust regular expression syntax, which is almost
identical to perl/python with a few minor differences

https://doc.rust-lang.org/regex/regex/index.html#syntax.\n\n    ")]
    pub fields: String,

    #[structopt(short="l", long="long")]
    /// Print items in the 'long form'
    pub long: bool,

    #[structopt(short="s", long="spc", default_value=">0", help = "\
Filter by spc (specification) completeness
- `-s \"<45\"`: show only items with spc <= 45%.
- `-s \">45\"`: show only items with spc >= 45%.
- `-s \"<\"`  : show only items with spc <=  0%.
- `-s \">\"`  : show only items with spc >=100%\n\n    ")]
    pub spc: String,

    #[structopt(short="t", long="tst", default_value=">0")]
    /// Filter by tst (test) completeness. See `-s/--spc` for format.
    pub tst: String,

    #[structopt(short="N", long="name")]
    /// \"name\" field: show the name of the artifact.
    pub name: bool,

    #[structopt(short="F", long="file")]
    /// \"file\" field: show the file where the artifact is defined.
    pub file: bool,

    #[structopt(short="S", long="subnames")]
    /// \"subnames\" field: show the subnames of the artifact.
    pub subnames: bool,

    #[structopt(short="P", long="parts")]
    /// \"parts\" field: show the children of the artifact.
    pub parts: bool,

    #[structopt(short="O", long="partof")]
    /// \"partof\" field: show the parents of the artifact.
    pub partof: bool,

    #[structopt(short="I", long="impl")]
    /// \"impl\" field: show the where the artifact is implemented.
    pub impl_: bool,

    #[structopt(short="T", long="text")]
    /// \"text\" field: show the text of the artifact.
    pub text: bool,

    #[structopt(short="A", long="all")]
    /// \"all\" field: activate ALL fields, additional fields DEACTIVATE fields.
    pub all: bool,

    #[structopt(long="plain")]
    /// Do not display color in the output.
    pub plain: bool,

    #[structopt(long="type", default_value="list")]
    /// Type of output from [list, json]
    pub output_ty: String,
}

/// Run the `art ls` command
pub fn run(cmd: Ls) -> Result<i32> {
    let mut w = io::stdout();

    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-ls in repo {}", repo.display());

    let (_, project) = read_project(repo)?;
    let display_flags = Flags::from_cmd(&cmd);
    let mut filtered = filter_artifacts(&cmd, &project.artifacts)?;
    filtered.sort();

    let ty_ = OutputType::from_str(&cmd.output_ty)?;
    if ty_ == OutputType::Json {
        let artifacts: Vec<_> = filtered.iter().map(|n| project.artifacts.get(n)).collect();
        write!(w, "{}", expect!(json::to_string_pretty(&artifacts)))?;
        return Ok(0);
    }

    if cmd.long {
        for name in filtered.iter() {
            let art = &project.artifacts[name];
            for el in &mut art.full_style(&project.artifacts, &display_flags) {
                if cmd.plain {
                    el.set_plain();
                }
                el.paint(&mut w)?;
            }
        }
    } else {
        display_table(&mut w, &cmd, &display_flags, &filtered, &project.artifacts)?;
    }

    Ok(0)
}

fn filter_artifacts(cmd: &Ls, artifacts: &OrderMap<Name, Artifact>) -> Result<OrderSet<Name>> {
    let fields = Flags::from_str(&cmd.fields)?;
    ensure!(!fields.impl_, "I/impl field not supported in search");
    let re = match cmd.pattern {
        Some(ref p) => {
            if p.starts_with("(?") {
                Regex::new(p)
            } else {
                // ignore case by default
                Regex::new(&format!("(?i){}", p))
            }?
        }
        None => return Ok(artifacts.keys().cloned().collect()),
    };
    // return true if we should keep
    let filter_map = |(name, art): (&Name, &Artifact)| -> Option<Name> {
        debug_assert_eq!(name, &art.name);
        macro_rules! check { [$field:expr] => {{
            if !re.is_match($field) {
                return None;
            }
        }}}

        if fields.name {
            check!(art.name.as_str());
        }
        if fields.file {
            check!(&art.file.to_string_lossy())
        }
        if fields.parts {
            if art.parts.iter().all(|n| !re.is_match(n.as_str())) {
                return None;
            }
        }
        if fields.partof {
            if art.partof.iter().all(|n| !re.is_match(n.as_str())) {
                return None;
            }
        }
        if fields.text {
            check!(&art.text);
        }

        Some(name.clone())
    };

    Ok(artifacts.iter().filter_map(filter_map).collect())
}

/// SPC-cli-ls.table
fn display_table<W: IoWrite>(
    w: &mut W,
    cmd: &Ls,
    display_flags: &Flags,
    filtered: &OrderSet<Name>,
    artifacts: &OrderMap<Name, Artifact>,
) -> io::Result<()> {
    let mut header = vec![vec![t!("spc%").bold()], vec![t!("tst%").bold()]];
    if display_flags.name {
        header.push(vec![t!(" | name").bold()]);
    }
    if display_flags.subnames {
        header.push(vec![t!(" | subnames").bold()]);
    }
    if display_flags.parts {
        header.push(vec![t!(" | parts").bold()]);
    }
    if display_flags.partof {
        header.push(vec![t!(" | partof").bold()]);
    }
    if display_flags.file {
        header.push(vec![t!(" | file").bold()]);
    }
    if display_flags.impl_ {
        header.push(vec![t!(" | impl").bold()]);
    }
    if display_flags.text {
        header.push(vec![t!(" | text").bold()]);
    }
    let mut rows = Vec::with_capacity(filtered.len() + 1);
    rows.push(header);

    rows.extend(
        filtered
            .iter()
            .map(|name| artifacts[name].line_style(artifacts, &display_flags)),
    );
    let mut table = El::Table(Table::new(rows));
    if cmd.plain {
        table.set_plain();
    }
    table.paint(w)
}

#[derive(Debug, Eq, PartialEq)]
enum OutputType {
    List,
    Json,
}

impl OutputType {
    fn from_str(s: &str) -> Result<OutputType> {
        Ok(match s {
            "list" => OutputType::List,
            "json" => OutputType::Json,
            _ => bail!("Invalid output type: {}", s),
        })
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Flags {
    name: bool,
    file: bool,
    subnames: bool,
    parts: bool,
    partof: bool,
    impl_: bool,
    text: bool,
}

lazy_static!{
    pub static ref VALID_SEARCH_FIELDS: OrderSet<&'static str> = OrderSet::from_iter(
        ["N", "F", "S", "P", "O", "I", "T", "A",
        "name", "file", "subnames", "parts", "partof", "impl", "text", "all"]
        .iter().map(|s| *s));

    pub static ref ANY_UPPERCASE: Regex = Regex::new("[A-Z]").unwrap();
}

impl Default for Flags {
    fn default() -> Flags {
        let mut out = Flags::empty();
        out.name = true;
        out.parts = true;
        out
    }
}

impl Flags {
    pub fn empty() -> Flags {
        Flags {
            name: false,
            file: false,
            subnames: false,
            parts: false,
            partof: false,
            impl_: false,
            text: false,
        }
    }

    pub fn from_str<'a>(s: &'a str) -> Result<Flags> {
        ensure!(!s.is_empty(), "Must search at least one field");
        let flags: OrderSet<&'a str> = if s.contains(',') {
            s.split(',').filter(|s| !s.is_empty()).collect()
        } else if !ANY_UPPERCASE.is_match(s) {
            orderset!(s)
        } else {
            s.split("").filter(|s| !s.is_empty()).collect()
        };

        let invalid: OrderSet<&'a str> =
            flags.difference(&VALID_SEARCH_FIELDS).map(|s| *s).collect();
        ensure!(invalid.is_empty(), "Unknown fields: {:#?}", invalid);
        let fc = |c| flags.contains(c);
        let all = fc("A") || fc("all");
        let out = Flags {
            name: fc("N") || fc("name"),
            file: fc("F") || fc("file"),
            subnames: fc("S") || fc("subnames"),
            parts: fc("P") || fc("parts"),
            partof: fc("O") || fc("partof"),
            impl_: fc("I") || fc("impl"),
            text: fc("T") || fc("text"),
        };
        Ok(out.resolve_actual(all))
    }

    /// Get the given flags from the command
    pub fn from_cmd(cmd: &Ls) -> Flags {
        let out = Flags {
            name: cmd.name,
            file: cmd.file,
            subnames: cmd.subnames,
            parts: cmd.parts,
            partof: cmd.partof,
            impl_: cmd.impl_,
            text: cmd.text,
        };
        let out = if cmd.long && !cmd.all && out.len() == 0 {
            // For the "long" form we display everything by default
            Flags {
                name: true,
                file: true,
                subnames: true,
                parts: true,
                partof: true,
                impl_: true,
                text: true,
            }
        } else {
            out
        };
        out.resolve_actual(cmd.all)
    }

    /// Flags with `all` taken into account.
    ///
    /// If no flags are set then use the default.
    fn resolve_actual(self, all: bool) -> Flags {
        if all {
            self.invert()
        } else if self.len() == 0 {
            Flags::default()
        } else {
            self
        }
    }

    /// Return the number of flags set
    pub fn len(&self) -> usize {
        macro_rules! add { [ $( $x:expr ),* ] => {{
            let mut out = 0;
            $( out += $x as usize; )*
            out
        }}}
        add!(
            self.name,
            self.file,
            self.subnames,
            self.parts,
            self.partof,
            self.impl_,
            self.text
        )
    }

    /// Invert the flag selection.
    fn invert(&self) -> Flags {
        Flags {
            name: !self.name,
            file: !self.file,
            subnames: !self.subnames,
            parts: !self.parts,
            partof: !self.partof,
            impl_: !self.impl_,
            text: !self.text,
        }
    }
}

/// Faster `Text`
trait ArtifactExt {
    fn line_style(&self, artifacts: &OrderMap<Name, Artifact>, flags: &Flags) -> Vec<Vec<Text>>;

    fn full_style(&self, artifacts: &OrderMap<Name, Artifact>, flags: &Flags) -> Vec<El>;

    fn name_style(&self) -> Text;

    fn subname_style(&self, subname: &SubName) -> Text;
}

impl ArtifactExt for Artifact {
    fn line_style(&self, artifacts: &OrderMap<Name, Artifact>, flags: &Flags) -> Vec<Vec<Text>> {
        let mut out = Vec::with_capacity(flags.len() + 2);
        macro_rules! cell { [$item:expr] => {{
            let mut cell = $item;
            cell.insert(0, t!(" | "));
            out.push(cell);
        }}};

        out.push(vec![self.completed.spc_style()]);
        out.push(vec![self.completed.tst_style()]);

        if flags.name {
            cell!(vec![self.name_style()])
        }
        if flags.subnames {
            let mut styles = Vec::new();
            for s in self.subnames.iter() {
                styles.push(self.subname_style(s));
                styles.push(t!(", "));
            }
            if !styles.is_empty() {
                styles.pop(); // remove trailing comma
            }
            cell!(styles);
        }
        if flags.parts {
            cell!(lookup_name_styles(artifacts, &self.parts));
        }
        if flags.partof {
            cell!(lookup_name_styles(artifacts, &self.partof));
        }
        if flags.file {
            cell!(vec![t!(self.file.display().to_string())]);
        }
        if flags.impl_ {
            cell!(vec![t!(self.impl_.to_string())]);
        }
        if flags.text {
            cell!(vec![t!(truncate(&self.text, 30).replace("\n", "\\n"))]);
        }
        out
    }

    fn full_style(&self, artifacts: &OrderMap<Name, Artifact>, flags: &Flags) -> Vec<El> {
        let mut out = Vec::new();

        macro_rules! line { [ $( $x:expr ),* ] => {{
            $( out.push(El::Text($x)); )*
            out.push(El::Text(t!("\n")));
        }}}

        macro_rules! extend_names { [ $title:expr, $x:expr ] => {{
            line![t!(concat!($title, ":")).bold()];
            for name in lookup_name_styles(artifacts, &$x) {
                line![t!("- ").bold(), name];
            }
        }}}

        // Name and completion
        line![t!("# ").bold(), self.name_style().bold()];
        line![
            t!("Completed: spc=").bold(),
            self.completed.spc_style(),
            t!("%  tst=").bold(),
            self.completed.tst_style(),
            t!("%").bold()
        ];

        if flags.file {
            line![t!("File: ").bold(), t!(self.file.display().to_string())];
        }
        if flags.impl_ {
            line![t!("Implemented: ").bold(), t!(self.impl_.to_string())];
        }
        if flags.parts {
            extend_names!("Parts", self.parts);
        }
        if flags.partof {
            extend_names!("Partof", self.partof);
        }
        if flags.text {
            line![t!(self.text.trim_right().to_string())]
        }

        line![t!("\n\n")];

        out
    }

    fn name_style(&self) -> Text {
        t!(self.name.as_str()).color(self.completed.name_color())
    }

    fn subname_style(&self, sub: &SubName) -> Text {
        let color = match self.impl_ {
            Impl::Done(_) => Red,
            Impl::Code(ref code) => {
                if code.secondary.contains_key(sub) {
                    Green
                } else {
                    Red
                }
            }
            Impl::NotImpl => Red,
        };
        t!(sub.as_str()).color(color)
    }
}

/// Truncate a string up to a certain number of _characters_.
fn truncate(s: &str, len: usize) -> String {
    let mut out = String::new();
    for c in s.chars().take(len) {
        out.push(c);
    }
    out
}

/// Find the styles of names that may or may not exist.
fn lookup_name_styles(artifacts: &OrderMap<Name, Artifact>, names: &OrderSet<Name>) -> Vec<Text> {
    let lookup = |name: &Name| match artifacts.get(name) {
        None => t!(name.as_str()).italic(),
        Some(art) => art.name_style(),
    };

    let mut out = Vec::new();
    for name in names {
        out.push(lookup(name));
        out.push(t!(", "));
    }
    if !out.is_empty() {
        out.pop(); // remove trailing comma
    }
    out
}

trait CompletedExt {
    fn spc_style(&self) -> Text;
    fn spc_points(&self) -> u8;
    fn tst_style(&self) -> Text;
    fn tst_points(&self) -> u8;
    fn name_color(&self) -> Color;
}

impl CompletedExt for Completed {
    /// #SPC-cli-ls.color_spc
    fn spc_style(&self) -> Text {
        let color = match self.spc_points() {
            0 => Red,
            1 => Yellow,
            2 => Blue,
            3 => Green,
            _ => unreachable!(),
        };
        t!(format!("{:.1}", self.spc * 100.0)).color(color)
    }

    fn spc_points(&self) -> u8 {
        if self.spc >= 1.0 {
            3
        } else if self.spc >= 0.7 {
            2
        } else if self.spc >= 0.4 {
            1
        } else {
            0
        }
    }

    /// #SPC-cli-ls.color_tst
    fn tst_style(&self) -> Text {
        let color = match self.tst_points() {
            0 => Red,
            1 => Yellow,
            2 => Green,
            _ => unreachable!(),
        };
        t!(format!("{:.1}", self.tst * 100.0)).color(color)
    }

    fn tst_points(&self) -> u8 {
        if self.tst >= 1.0 {
            2
        } else if self.tst >= 0.5 {
            1
        } else {
            0
        }
    }

    /// #SPC-cli-ls.color_name
    fn name_color(&self) -> Color {
        match self.spc_points() + self.tst_points() {
            0 => Red,
            1 | 2 => Yellow,
            3 | 4 => Blue,
            5 => Green,
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_flags_str() {
    let mut flags = Flags::default();
    macro_rules! from_str { ($f:expr) => {{
        expect!(Flags::from_str($f))
    }}}
    assert_eq!(flags, from_str!("NP"));
    assert_eq!(flags, from_str!("N,parts"));
    assert_eq!(flags, from_str!("name,parts"));
    assert_eq!(flags, from_str!("AFOITS"));
    flags.text = true;
    assert_eq!(flags, from_str!("NTP"));
    assert_eq!(flags, from_str!("TNP"));
    assert_eq!(flags, from_str!("text,parts,name"));
    flags.parts = false;
    flags.text = false;
    assert_eq!(flags, from_str!("N"));
    assert_eq!(flags, from_str!("name"));

    assert!(Flags::from_str("").is_err());
}

#[test]
fn test_style() {
    {
        let completed = Completed {
            spc: 0.33435234,
            tst: 1.0,
        };
        assert_eq!(t!("33.4").color(Red), completed.spc_style());
        assert_eq!(t!("100.0").color(Green), completed.tst_style());
    }

    {
        let completed = Completed {
            spc: 0.05,
            tst: 0.0,
        };
        assert_eq!(t!("5.0").color(Red), completed.spc_style());
        assert_eq!(t!("0.0").color(Red), completed.tst_style());
    }

    let art = Artifact {
        id: HashIm::default(),
        name: name!("REQ-foo"),
        file: PathArc::new("/fake"),
        partof: orderset!{},
        parts: orderset!{},
        completed: Completed {
            spc: 1.0,
            tst: 0.003343,
        },
        text: "some text".into(),
        impl_: Impl::NotImpl,
        subnames: orderset!{},
    };
    let artifacts = ordermap![
        name!("REQ-foo") => art.clone(),
    ];

    let expected = vec![
        // % spc+tst completed
        vec![t!("100.0").color(Green)],
        vec![t!("0.3").color(Red)],
        // name
        vec![t!(" | "), t!("REQ-foo").color(Blue)],
        // parts
        vec![t!(" | ")],
    ];
    let flags = Flags::default();
    assert_eq!(expected, art.line_style(&artifacts, &flags));
}
