#[cfg(feature = "web")] extern crate tar;
extern crate serde_codegen;

pub use std::env;
pub use std::path::Path;
pub use std::process::Command;


#[cfg(feature = "web")] 
mod web {
    use std::io::Read;
    use super::*;
    use std::fs;

    use tar;

    #[cfg(windows)]
    fn build_web_ui(_: &Path) {
        println!("not building web-ui because we are on windows");
    }

    #[cfg(not(windows))]
    fn build_web_ui(web_dir: &Path) {
        let npm_build = Command::new("npm")
            .current_dir(&web_dir)
            .args(&["run", "build"])
            .output()
            .expect("failed to build web-ui");
        println!("npm build succeeded:\n{:?}", npm_build.stdout);
    }

    pub fn package_web_ui() {
        let cwd = env::current_dir().unwrap();
        let web_dir = cwd.join("web-ui");
        build_web_ui(&web_dir);

        let dist_dir = web_dir.join("dist");
        let tarfile = cwd.join("target").join("web-ui.tar");

        fs::remove_file(&tarfile).ok();
        let tarfile_fp = fs::File::create(&tarfile).expect("could not create tarfile");
        let mut packager = tar::Builder::new(tarfile_fp);
        let dist = fs::read_dir(dist_dir).expect("could not open web dist");
        for pack_file_path in dist {
            let pack_file_path = pack_file_path.unwrap().path();
            let mut file = fs::File::open(&pack_file_path).expect("could not open dist file");
            let mut data: Vec<u8> = Vec::new();
            file.read_to_end(&mut data).unwrap();

            let mut header = tar::Header::new_old();
            header.set_path(&pack_file_path.file_name().unwrap()).unwrap();
            header.set_size(data.len() as u64);
            header.set_cksum();
            packager.append(&header, data.as_slice()).unwrap();
        }
    }
}

#[cfg(not(feature = "web"))] 
mod web {
    pub fn package_web_ui() {
    }
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/core/serde_types.in.rs");
    let dst = Path::new(&out_dir).join("serde_types.rs");
    serde_codegen::expand(&src, &dst).unwrap();
    web::package_web_ui();
}
