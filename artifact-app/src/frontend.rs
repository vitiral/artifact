/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
//! Helper methods for managing the frontend.
use dev_prelude::*;

use tar::Archive;
use tempdir::TempDir;

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("../../target/frontend.tar");

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
