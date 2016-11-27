//! This whole file was pretty much copy/pasted from
//! https://github.com/alexcrichton/toml-rs/
//! folder src/display.rs
//! 
//! This file is therefore Licensed under the MIT
//! License, not the license of rst
//! 
//! MIT License
//! Copyright (c) 2016 Garrett Berg
//! Copyright (c) 2014 Alex Crichton

use std::fmt::Write;
use std::fmt;
use toml::{Value, Table};

/// print a toml Table prettily -- strings with newlines are printed
/// with the tripple quote syntax
pub fn pretty_toml<'a>(tbl: Table) -> Result<String, fmt::Error> {
    let mut out = String::new();
    {
        let mut pp = PrettyPrinter { output: &mut out, stack: Vec::new() };
        try!(pp.print(&tbl));
    }
    Ok(out)
}

fn write_pretty_str(f: &mut String, s: &str) -> fmt::Result {
    try!(write!(f, "'''\n"));
    for ch in s.chars() {
        match ch {
            '\u{8}' => try!(write!(f, "\\b")),
            '\u{9}' => try!(write!(f, "\\t")),
            '\u{c}' => try!(write!(f, "\\f")),
            '\u{d}' => try!(write!(f, "\\r")),
            '\u{22}' => try!(write!(f, "\\\"")),
            '\u{5c}' => try!(write!(f, "\\\\")),
            ch => try!(write!(f, "{}", ch)),
        }
    }
    write!(f, "'''")
}

// The only thing in this impl that wasn't copy/pasted is
// - I removed handling arrays of tables (panics)
// - I added pretty printing strings when the have a \n in them
impl<'a, 'b> PrettyPrinter<'a, 'b> {
    fn print(&mut self, table: &'a Table) -> fmt::Result {
        let mut space_out_first = false;
        // print out the regular key/value pairs at the top,
        // including arrays of tables I guess? (who cares)
        for (k, v) in table.iter() {
            match *v {
                Value::Table(..) => continue,
                Value::Array(ref a) => {
                    if let Some(&Value::Table(..)) = a.first() {
                        // not supported in rst
                        panic!("attempting to serialize an array of tables!")
                    }
                }
                // super special case -- the whole reason this is here!
                Value::String(ref s) => {
                    if s.contains('\n') {
                        try!(write!(self.output, "{} = ", Key(&[k])));
                        try!(write_pretty_str(self.output, s));
                        try!(write!(self.output, "\n"));
                        space_out_first = true;
                        continue;
                    }
                }
                _ => {}
            }
            space_out_first = true;
            try!(writeln!(self.output, "{} = {}", Key(&[k]), v));
        }
        // now go through the table and format the other tables
        for (i, (k, v)) in table.iter().enumerate() {
            match *v {
                Value::Table(ref inner) => {
                    // store the stack so that we can write
                    // [table.foo.bar]
                    self.stack.push(k);
                    if space_out_first || i != 0 {
                        try!(write!(self.output, "\n"));
                    }
                    try!(writeln!(self.output, "[{}]", Key(&self.stack)));
                    try!(self.print(inner));
                    self.stack.pop();
                }
                _ => {},
            }
        }
        Ok(())
    }
}

/// pretty printer for making multi-line text prettier
/// uses a String instead of the formatter from before
struct PrettyPrinter<'a, 'b:'a> {
    output: &'b mut String,
    stack: Vec<&'a str>,
}

// Everything below this line (until the tests) is a direct copy/paste
// from toml-rs

struct Key<'a>(&'a [&'a str]);

fn write_str(f: &mut fmt::Formatter, s: &str) -> fmt::Result {
    try!(write!(f, "\""));
    for ch in s.chars() {
        match ch {
            '\u{8}' => try!(write!(f, "\\b")),
            '\u{9}' => try!(write!(f, "\\t")),
            '\u{a}' => try!(write!(f, "\\n")),
            '\u{c}' => try!(write!(f, "\\f")),
            '\u{d}' => try!(write!(f, "\\r")),
            '\u{22}' => try!(write!(f, "\\\"")),
            '\u{5c}' => try!(write!(f, "\\\\")),
            ch => try!(write!(f, "{}", ch)),
        }
    }
    write!(f, "\"")
}

impl<'a> fmt::Display for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, part) in self.0.iter().enumerate() {
            if i != 0 { try!(write!(f, ".")); }
            let ok = part.chars().all(|c| {
                match c {
                    'a' ... 'z' |
                    'A' ... 'Z' |
                    '0' ... '9' |
                    '-' | '_' => true,
                    _ => false,
                }
            });
            if ok {
                try!(write!(f, "{}", part));
            } else {
                try!(write_str(f, part));
            }
        }
        Ok(())
    }
}

// #############################################################################
// Tests


#[test]
fn test_pretty() {
    // examples of the form (input, expected output). If expected output==None, 
    // then it == input
    let mut examples = vec![
// toml keeps pretty strings
(r##"[example]
a_first = "hello world"
b_second = '''
this is a little longer
yay, it looks good!
'''
"##, None),

// format with two tables
(r##"[a_first]
int = 7
long = '''
i like long text
it is nice
'''

[b_second]
int = 10
text = "this is some text"
"##, None),

// toml re-orders fields alphabetically
(r##"[example]
b_second = ''' woot '''
a_first = "hello world"
"##, 
Some(r##"[example]
a_first = "hello world"
b_second = " woot "
"##)),

// toml reorders tables alphabetically
("[b]\n[a]\n", Some("[a]\n\n[b]\n")),
];

    use super::tests::parse_text;
    for (i, (value, expected)) in examples.drain(..).enumerate() {
        let expected = match expected {
            Some(ref r) => r,
            None => value,
        };
        assert_eq!((i, pretty_toml(parse_text(value)).unwrap()), (i, expected.to_string()));
    }
}
