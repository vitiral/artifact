//! Helper methods for managing the frontend.
use dev_prelude::*;

use tar::Archive;
use tempdir::TempDir;

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("../artifact-frontend/target/frontend.tar");

/// Unpack the web frontend, overriding the initial project settings.
pub(crate) fn unpack_frontend<P: AsRef<Path>>(
    into: P,
    init: &ProjectInitialSer,
) -> ::std::io::Result<()> {
    let dir = into.as_ref();
    info!("Unpacking frontend at: {}", dir.display());

    let mut archive = Archive::new(WEB_FRONTEND_TAR);
    archive.unpack(dir)?;

    let init_s = expect!(json::to_string(init));

    let init_f = PathFile::new(dir.join("initial.json"))?;
    init_f.write_str(&init_s)?;

    Ok(())
}
