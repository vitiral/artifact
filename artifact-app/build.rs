#[macro_use]
extern crate ergo;
use ergo::*;
use std::process::Command;

fn main() {
    println!("Build Script Started");
    check_deps();
    // build_frontend();
    build_mdbook();
    cp_mdbook();
    tar_frontend().expect("tarring frontend failed");
}

lazy_static! {
    static ref WORKSPACE: PathDir = PathDir::new("..").unwrap();
    static ref FRONTEND: PathDir = PathDir::new("../artifact-frontend").unwrap();
    static ref BOOK: PathDir = PathDir::new("../book").unwrap();
    static ref TARGET: PathDir = PathDir::create_all(WORKSPACE.join("target")).unwrap();
    static ref FRONTEND_DEPLOY: PathDir = PathDir::create_all(TARGET.join("deploy")).unwrap();
}

fn check_deps() {
    println!("Checking dependencies");

    let mut missing = Vec::new();

    let check_cmd = |m: &mut Vec<_>, cmd: &'static str| {
        let is_ok = Command::new("which")
            .args(&[cmd])
            .output()
            .expect("which/where doesn't exist")
            .status
            .success();

        if !is_ok {
            m.push(cmd);
        }
    };

    check_cmd(&mut missing, "mdbook");

    if !missing.is_empty() {
        println!("ERROR: Missing binary dependencies, their binaries must be put in target/deps/");
        for c in missing {
            println!("  {}", c);
        }
        ::std::process::exit(1);
    } else {
        println!("- All dependencies found");
    }
}

fn build_mdbook() {
    println!("Building the book");
    let _status = Command::new("mdbook")
        .current_dir(BOOK.as_path())
        .args(&["build"])
        .status()
        .unwrap();
}

fn cp_mdbook() {
    println!("Copying mdbook to deploy");
    let (send, recv) = ch::unbounded();
    let deploy = PathDir::create_all(FRONTEND_DEPLOY.join("docs")).unwrap();
    deploy.clone().remove_all().unwrap();
    deep_copy(
        send,
        PathDir::new(BOOK.join("out").join("book")).unwrap(),
        deploy,
    );
    let errs: Vec<_> = recv.into_iter().collect();
    assert!(errs.is_empty(), "{:?}", errs);
}

// fn build_frontend() {
//     println!("Building artifact-frontend");
//     let status = Command::new("cargo-web")
//         .current_dir(WORKSPACE.as_path())
//         .args(&[
//             "deploy",
//             "-p",
//             "artifact-frontend",
//             "--target=wasm32-unknown-unknown",
//             "--release",
//         ])
//         .status()
//         .unwrap();
//
//     assert!(status.success(), "artifact-frontend failed to build");
// }

fn tar_frontend() -> ::std::io::Result<PathFile> {
    let target = PathDir::new(WORKSPACE.join("target"))?;
    let deploy = match PathDir::new(target.join("deploy")) {
        Ok(d) => d,
        Err(e) => panic!("target/deploy doesn't exist. justfile::build-frontend defines how."),
    };
    let archive_path = PathFile::create(target.join("frontend.tar"))?;

    println!("Taring frontend {:?} into {:?}", deploy, archive_path);
    let mut f = FileWrite::create(&archive_path)?;
    let mut builder = tar::Builder::new(&mut f);

    tar_dir(&deploy, &deploy, &mut builder)?;

    builder.finish()?;

    Ok(archive_path)
}

fn tar_dir<W: ::std::io::Write>(
    prefix: &PathDir,
    from: &PathDir,
    builder: &mut tar::Builder<W>,
) -> ::std::io::Result<()> {
    for entry in from.list()? {
        let file = match entry.unwrap() {
            PathType::File(f) => f,
            PathType::Dir(d) => {
                tar_dir(prefix, &d, builder);
                continue;
            }
        };
        let mut header = tar::Header::new_old();
        header.set_metadata_in_mode(&file.metadata()?, tar::HeaderMode::Deterministic);
        header.set_path(&file.strip_prefix(prefix).unwrap())?;
        header.set_cksum();
        let mut r = file.open_read()?;
        builder.append(&header, &mut r)?;
    }
    Ok(())
}
