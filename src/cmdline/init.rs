use std::io;
use std::io::Write;
use std::fs;
use std::path::Path;
use std::collections::HashSet;

use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS};

use core;

const SETTINGS_RSK: &'static str = r#"\
# This is an artifacts file for the command line tool rsk
# files (like this one) that are in {repo}/.rsk are automatically loaded

[settings]
# Any *.rsk file can define additional paths to be loaded by listing them
# in a path.
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file
# - {repo}: the path to the current repository, which is the closest
#    directory that contains a ".rsk" folder
## TODO: uncomment this line and define your own paths
# paths = ['{repo}/docs']

## LEARN: try uncommenting out this line and then type `rsk ls`
# [REQ-example]
"#;

const TUTORIAL_RSK: &'static str = r#"\

# Welcome to the rsk tutorial! This file is written in a way that
# your .rsk files can be written in to define your requirements
# and design specifications.
#
# rsk is a command line, text based tool for requriements tracking
# it aims to be easy to use by anybody and extremely productive
# for developers especially
#
# Follow along with this file to get an idea of how you can
# easily write and track requirements
#
# before we continue too far, why don't you try a command? Type
# the following command:
#
#     rsk ls SPC-learn-ls -l
#
# You should see a colorized artifact that is a partof of a few other
# artifacts. If you do, good. If you are having problems try going back to
# the installation tutorial.
# Run `rsk ls -h` and try a few other commands to get a feel for the interface.

##################################################
# Defining requirements

[REQ-toml]
text = '''
.rsk files like this one are written in the TOML format
you can read more about it here: https://github.com/toml-lang/toml

all rsk files must end in "rsk"

They are all flat. This means rsk does not support the "first.second]"
syntax. This means that the " character is illegal in names
'''

[REQ-learn-rsk]
text = '''
This is what is known as an "artifact"

Artifacts can be a requirement (REQ), design-specification (SPC)
risk (RSK) or test (TST)

This particular artifact is a requirement, therefore it begins with
"REQ" After REQ there is a " and then the name of the requirement.

Unlike many requirements tracking tools, rsk encourages the use
of human-readable names. We will talk more about this in the next
artifacts.
'''

[SPC-learn-spc]
text = '''
This is a design specification and is automatically linked as a "partof"
REQ-learn-rsk. In addition, it is a partof REQ-toml because it was explicitly
linked using the "partof" field.
'''

[REQ-learn-partof]
text = "see the next artifact"

[SPC-learn-partof]
partof = "REQ-learn-rsk"
text = '''
RSK uses the names of artifacts to automatically link them and track progress
as well as makes it simple for the user to link to arbitrary artifacts.
(but they must be of specific types, more on that later)

REQ-learn-rsk is explicitly a "partof" this artifact because it is specified
explicitly.

REQ-learn-partof is automatically a partof this artifact because the names
after the type are the same.

In addition, missing parents are automatically created and linked. So
SPC-LEARN is also a partof this artifact, even though it is not even in this
document. This makes it very easy to make trees of artifacts without needing to
specify every branch.

So far we have:

REQ <-- REQ-LEARN <-- REQ-LEARN-PARTOF <-- SPC-learn-partof
           ^                                  |
            \\---------REQ-learn-partof <------/

(Note: only parents are created automatically. Auto-creating for similar-named
  artifacts would polute your links)
'''

[SPC-learn-loc]
loc = "repo}/example.py"
text = '''
you can mark that an artifact is 100% "completed" by setting the "loc"
variable. This will look in the given file for the artifact name
(in this case SPC-learn-loc), and if it finds it it will consider the
artifact done.

when `rsk ls` is called, the percent completed and tested of each
artifact will be displayed and colorized.

If a SPC or TST has `loc` set, it is considered completed. Otherwise,
the values are calculated based on the average of their "parts"
(parts is the opposite of partof)
'''

[TST-learn-partof]
partof = "SPC-learn-[spc, loc]"
text = '''
The partof field is a string that uses a simple grouping syntax. This example
does as you would expect, this artifact is a partof SPC-learn-spc and SPC-learn-loc

note: it is also automatically a partof SPC-learn-partof and TST-LEARN because of the name
'''

[TST-learn-valid]
text = '''
There are some rules for which artifacts can be a partof other artifacts:
- all artifacts can be a partof their own group (i.e. SPC partof SPC)
- SPC can be a partof REQ
- RSK can be a partof REQ
- TST can be a partof SPC or RSK
- REQ can only be a partof itself

Here is a helpful graph of valid relations:
  REQ <-- SPC* <-- TST*
   ^                |
    \\---- RSK <-----/

In other words, you can design a spec (SPC) based on
a requirement (REQ). A requirement can have a risk (RSK)
associated with it. Tests can test to either a spec (SPC)
or to a risk (RSK)

[SPC-learn-paths]
text = '''
in the {repo}/.rsk/settings.rsk file you must define the path where you want
to put your artifacts. This path will be searched recursively but will stop
at the first directory where there is not a .rsk file.

There are only a few rules for defining artifacts:
 - case is ignored for all names (except globals and settings)
 - names cannot overlap, even in different files
 - artifacts must follow TST-learn-valid (see above)
 - all items (i.e. [REQ-foo]) must start with either REQ, RSK, SPC or TST
     *except* for "globals""and "settings"
'''

[SPC-learn-ls]
text = '''
The `ls` command is the most important component to learn in rsk, as it helps you
manage the artifacts in your project, view how completed/tested they are,
see whether they are linked correctly, etc.

Type:
    rsk ls SPC-learn-ls -l

This will show you this artifact, pretty printed on your terminal.

Try:
    rsk ls learn -p N

This searches in the "Name" field for all artifacts that have "learn"
in their name.

Let's say you wanted to find an artifact, and all you knew was that it mentioned files
in it's description. You could run:

    rsk ls file -p T -T

This will search for the pattern "file" in the text field (`-p T`). It will also display a
short piece of the text field.

You see that SPC-learn-paths is what you were looking for, but you want an expanded view:
   rsk ls SPC-learn-paths -l

Now you see that SPC-learn-paths has not been tested or implemented and that it is partof
SPC-LEARN. From this you could decide to implement and test SPC-learn-paths.
'''

##################################################
# Extra stuff

[settings]
# Any *.rsk file can define additional paths to be loaded by listing them
# in a path.
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file
# - {repo}: the path to the current repository, which is the closest
#    directory that contains a "rsk" folder
# TODO: go to settings.rsk in this directory and follow it's dirrections

[globals]
# additional variables that can be used anywhere in the project
# can be defined like below.
example_var = "repo}/example"
# they can then be used in any text block like:
text = "my var: {example_var}"

##################################################
# That is the end of this tutorial. Run the following
# for part 2:
#     rsk init -t 2
"#;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // [SPC-ui-cmdline-cmd-init-interface]
    SubCommand::with_name("init")
        .about("initiailze the repository and get help")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("tutorial")
             .short("t")
             .help("also initialize the interactive tutorial"))
}

pub fn do_init(path: &Path, tutorial: bool) -> io::Result<()> {
    let mut read_dir = try!(fs::read_dir(path));
    let exists = read_dir.any(|e|
        match e {
                Err(_) => false,
                Ok(e) => {
                    if !e.file_type().unwrap().is_dir() {
                        false
                    } else {
                        let p = e.path();
                        let fname = p.file_name().unwrap().to_str().unwrap();
                        if fname == ".rsk" {
                            true
                        } else {
                            false
                        }
                    }
                }
        });
    let repo = path.join(".rsk");
    if !exists {
        try!(fs::create_dir(&repo));

        // create settings
        let settings = repo.join("settings.rsk");
        let mut f = try!(fs::File::create(settings));
        f.write_all(SETTINGS_RSK.as_ref()).unwrap();
    }


    // create tutorial
    if tutorial {
        let help = repo.join("tutorial.rsk");
        if exists {
            match fs::remove_file(&help) {
                _ => {} // if can't remove, don't care
            };
        }
        let mut f = try!(fs::File::create(help));
        f.write_all(TUTORIAL_RSK.as_ref()).unwrap();
        println!("See tutorial at {0}/tutorial.toml", repo.to_string_lossy().as_ref());
    } else {
        println!("rsk initialized at {0}", repo.to_string_lossy().as_ref());
    }
    Ok(())
}
