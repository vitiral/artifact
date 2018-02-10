//! basic high-level command tests

use dev_prefix::*;
use cmd;



#[test]
fn test_help() {
    let mut w: Vec<u8> = Vec::new();
    //let args = vec![OsString::from("art")];
    //cmd::cmd(&mut w, args.iter()).unwrap();

    let args = vec![OsString::from("art"), OsString::from("-h")];
    cmd::cmd(&mut w, args.iter()).unwrap();
}
