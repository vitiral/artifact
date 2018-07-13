#[macro_use]
extern crate ergo;
use ergo::*;
use std::process::Command;

fn main() {
    println!("Build Script Started");
    build_frontend();
    build_mdbook();
    cp_mdbook();
    tar_frontend().expect("tarring frontend failed");
}

lazy_static! {
    static ref FRONTEND: PathDir = PathDir::new("artifact-frontend").unwrap();
    static ref BOOK: PathDir = PathDir::new("book").unwrap();

    static ref FRONTEND_TARGET: PathDir = PathDir::new(FRONTEND.join("target")).unwrap();
    static ref FRONTEND_DEPLOY: PathDir = PathDir::new(FRONTEND_TARGET.join("deploy")).unwrap();
}

fn build_mdbook() {
    println!("Building the book");
    let status = Command::new("mdbook")
        .current_dir(BOOK.as_path())
        .args(&[
            "build",
        ])
        .status()
        .unwrap();
}

fn cp_mdbook() {
    println!("Copying mdbook to deploy");
    let (send, mut recv) = ch::unbounded();
    deep_copy(
        send,
        PathDir::new(BOOK.join("book")).unwrap(),
        FRONTEND_DEPLOY.join("docs"),
    );
    let errs: Vec<_> = recv.into_iter().collect();
    assert!(errs.is_empty(), "{:?}", errs);
}

fn build_frontend() {
    println!("Building artifact-frontend");
    let status = Command::new("cargo")
        .current_dir(FRONTEND.as_path())
        .args(&[
            "web",
            "deploy",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .status()
        .unwrap();

    assert!(status.success(), "artifact-frontend failed to build");
}

fn tar_frontend() -> ::std::io::Result<PathFile> {
    let target = PathDir::new(FRONTEND.join("target"))?;
    let deploy = PathDir::new(target.join("deploy"))?;
    let archive_path = PathFile::create(target.join("frontend.tar"))?;

    println!("Taring frontend {:?} into {:?}", deploy, archive_path);
    let mut f = FileWrite::create(&archive_path)?;
    let mut builder = tar::Builder::new(&mut f);

    tar_dir(&deploy, &deploy, &mut builder)?;

    builder.finish()?;

    Ok(archive_path)
}

fn tar_dir<W: ::std::io::Write>(prefix: &PathDir, from: &PathDir, builder: &mut tar::Builder<W>) -> ::std::io::Result<()> {
    for entry in from.list()? {
        let file = match entry.unwrap() {
            PathType::File(f) => f,
            PathType::Dir(d) => {
                tar_dir(prefix, &d, builder);
                continue;
            },
        };
        let mut header = tar::Header::new_old();
        header.set_metadata_in_mode(&file.metadata()?, tar::HeaderMode::Deterministic);
        header.set_path(&file.strip_prefix(prefix).unwrap())?;
        header.set_cksum();
        let mut r = file.read()?;
        builder.append(&header, &mut r)?;
    }
    Ok(())
}

