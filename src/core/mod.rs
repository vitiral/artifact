use std::path::Path;

use time;

pub mod types;
pub mod vars;
#[macro_use] pub mod load;  // macro use so the macro can be tested
pub mod link;
pub mod fmt;
pub mod prefix;

#[cfg(test)]
mod tests;

// export for other modules to use
pub use core::vars::find_repo;
pub use core::types::{
    LoadResult, LoadError,
    Artifacts, Artifact, ArtType, ArtName, ArtNames, Loc,
    Settings, LoadFromStr};
pub use core::load::load_toml;

#[cfg(test)]
use super::cmdline::init_logger;
#[cfg(test)]
pub fn init_logger_test() {
    match init_logger(false, 3, false) {
        Ok(_) => {},
        Err(_) => {},
    }
}



/// do all core loading operations defined in SPC-core-load-parts
/// includes loading and validating raw data, resolving and applying
/// variables, and linking artifacts
/// LOC-core-load-path
pub fn load_path(path: &Path) -> LoadResult<(Artifacts, Settings)>{
    let start = time::get_time();
    info!("loading path: {}", path.to_string_lossy().as_ref());
    let (mut artifacts, mut settings) = try!(load::load_path_raw(path));
    let locs = try!(vars::find_locs(&mut settings));
    vars::attach_locs(&mut artifacts, &locs);

    // LOC-core-load-parts-4:<auto-creation of missing prefix artifacts>
    link::link_named_partofs(&mut artifacts); // MUST come before parents are created
    link::create_parents(&mut artifacts);
    link::link_parents(&mut artifacts);

    // [#TST-core-artifact-attrs-partof-vaidate]
    try!(link::validate_partof(&artifacts));

    // LOC-core-load-parts-5:<linking of artifacts>
    link::link_parts(&mut artifacts);
    link::set_completed(&mut artifacts);
    link::set_tested(&mut artifacts);
    let total = time::get_time() - start;
    info!("Done loading: {} artifacts loaded successfullly in {:.3} seconds",
          artifacts.len(), total.num_milliseconds() as f64 * 1e-3);
    Ok((artifacts, settings))
}

