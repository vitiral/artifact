
use std::panic;
use dev_prefix::*;

use cmd::init;

use tempdir;


#[test]
/// #TST-cmd-init
fn test_init() {
    let tmpdir = tempdir::TempDir::new("artifact").unwrap();
    let dir = tmpdir.path();

    // basically try/finally for rust -- need to make sure we don't change
    // the actual data
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        init::run_cmd(&dir).expect("first init");
        let expected = "Expected";

        let settings = dir.join(".art").join("settings.toml");
        let purpose = dir.join("design").join("purpose.toml");

        assert!(settings.exists());
        assert!(purpose.exists());

        let write = |p| {
            fs::File::create(p)
                .expect("create")
                .write_all(expected.as_bytes())
                .expect("write")
        };
        let read = |p| {
            let mut out = String::new();
            fs::File::open(p)
                .expect("open")
                .read_to_string(&mut out)
                .expect("read");
            out
        };

        write(&settings);
        write(&purpose);

        // run init again and make sure nothing changed
        init::run_cmd(&dir).unwrap();
        assert_eq!(expected, read(&settings));
        assert_eq!(expected, read(&purpose));

        // delete the .art folder and run again, expect
        // the new settings file to exist but the old
        // purpose to remain
        fs::remove_dir_all(&dir.join(".art")).unwrap();
        init::run_cmd(&dir).unwrap();
        assert!(settings.exists());
        assert_eq!(expected, read(&purpose));

        // running init when there is already an .art folder does nothing
        fs::remove_dir_all(&dir.join("design")).unwrap();
        init::run_cmd(&dir).unwrap();
        assert!(!purpose.exists());
    }));
    drop(dir);
    result.unwrap();
}
