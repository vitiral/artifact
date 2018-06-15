#[macro_use]
extern crate ergo;
use ergo::*;
use std::process::Command;

fn main() {
    println!("Build Script Started");
    build_frontend();
    tar_frontend().expect("tarring frontend failed");
}

lazy_static! {
    static ref FRONTEND: PathDir = PathDir::new("artifact-frontend").unwrap();
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

    for file in deploy.list()? {
        let file = match file {
            Ok(PathType::File(f)) => f,
            _ => panic!("{:?}", file),
        };
        let mut header = tar::Header::new_old();
        header.set_metadata_in_mode(&file.metadata()?, tar::HeaderMode::Deterministic);
        header.set_path(&file.strip_prefix(&deploy).unwrap())?;
        header.set_cksum();
        let mut r = file.read()?;
        builder.append(&header, &mut r)?;
    }

    builder.finish()?;

    Ok(archive_path)
}
