use super::types::*;

lazy_static!{
    pub static ref SPC: VecDeque<char> = VecDeque::from_iter(vec!['#', 'S', 'P', 'C', '-']);
    pub static ref TST: VecDeque<char> = VecDeque::from_iter(vec!['#', 'T', 'S', 'T', '-']);
}

pub fn find_locs_text(path: &Path,
                      text: &str,
                      locs: &mut HashMap<ArtName, Loc>)
                      -> bool {
    let mut error = false;
    let text = text;
    let mut prev: VecDeque<char> = VecDeque::with_capacity(5);
    let mut prev_char = ' ';
    let mut start_pos = 0;
    let mut start_col = 0;
    let (mut pos, mut line, mut col) = (0, 1, 0); // line starts at 1
    // pretty simple parse tree... just do it ourselves!
    // Looking for #LOC-[a-z0-9_-] case insensitive
    for c in text.chars() {
        if prev == *SPC || prev == *TST {  // TODO: I'm sure this is not as fast as possible
            if prev_char == ' ' {
                start_pos = pos - 5;
                start_col = col - 5;
            }
            match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => {
                    prev_char = c;  // still reading a valid artifact name
                }
                _ => {  // valid #ART is finished
                    if prev_char != ' ' { // "SPC- ", etc is actually invalid
                        let art_start = start_pos + 1; // + 1 because of '#'
                        let (_, end) = text.split_at(art_start);
                        // if last char is '-' ignore it
                        let (name, _) = match prev_char {
                            '-' => end.split_at(pos - art_start - 1),
                            _ => end.split_at(pos - art_start),
                        };
                        let locname = ArtName::from_str(name).unwrap();
                        debug!("Found loc: {}", locname);
                        let loc = Loc {
                            path: path.to_path_buf(),
                            line_col: (line, start_col)
                        };
                        match locs.insert(locname, loc) {
                            None => {},
                            Some(l) => {
                                error!("detected overlapping loc {} in files: {:?} and {}",
                                        name, l, path.display());
                                error = true;
                            }
                        }
                        prev_char = ' ';
                    }
                    prev.pop_front();
                    prev.push_back(c);
                },
            }
        } else {
            if prev.len() == 5 {
                prev.pop_front();
            }
            prev.push_back(c);
        }
        match c {
            '\n' => {
                line += 1;
                col = 0;
            }
            _ => col += 1,
        };
        pos += 1;
    }
    error
}

/// [#SPC-core-load-loc-text]
/// given text, the path to the text, and the locations to add onto
/// extract all the locations from the text and return whether there
/// was an error
pub fn find_locs_file(path: &Path,
                      locs: &mut HashMap<ArtName, Loc>)
                      -> bool {
    debug!("resolving locs at: {:?}", path);
    let mut text = String::new();
    match fs::File::open(path) {
        Ok(mut f) => match f.read_to_string(&mut text) {
            Ok(_) => {},
            Err(e) => {
                error!("while reading from <{}>: {}", path.display(), e);
                return true;
            }
        },
        Err(e) => {
            error!("while loading from <{}>: {}", path.display(), e);
            return true;
        },
    }
    find_locs_text(path, &text, locs)
}

/// recursively find all locs given a directory
fn find_locs_dir(path: &PathBuf, loaded_dirs: &mut HashSet<PathBuf>,
                 locs: &mut HashMap<ArtName, Loc>)
                 -> bool {
    loaded_dirs.insert(path.to_path_buf());
    let read_dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(err) => {
            error!("while loading from dir <{}>: {}", path.display(), err);
            return true;
        }
    };
    let mut error = false;
    let mut dirs_to_load: Vec<PathBuf> = Vec::new(); // TODO: use references
    for entry in read_dir.filter_map(|e| e.ok()) {
        let fpath = entry.path();
        // don't parse .toml files for locations
        match fpath.extension() {
            None => {},
            Some(ext) => if ext == "toml" {
                continue
            }
        }
        let ftype = match entry.file_type() {
            Ok(f) => f,
            Err(err) => {
                error!("while loading from <{}>: {}", fpath.display(), err);
                error = true;
                continue;
            }
        };
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone());
        } else if ftype.is_file() {
            match find_locs_file(&fpath, locs) {
                true => error = true,
                false => {},
            }
        }
    };

    for d in dirs_to_load {
        if loaded_dirs.contains(&d) {
            continue;
        }
        match find_locs_dir(&d, loaded_dirs, locs) {
            true => error = true,
            false => {},
        }
    }
    error
}

/// search through the code_paths in settings to find all valid locs
/// partof: #SPC-loc
pub fn find_locs(settings: &mut Settings) -> LoadResult<HashMap<ArtName, Loc>> {
    info!("parsing code files for artifacts...");
    let mut locs: HashMap<ArtName, Loc> = HashMap::new();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::from_iter(
        settings.exclude_code_paths.iter().map(|p| p.to_path_buf()));
    // first make sure the excluded directories exist
    for d in loaded_dirs.iter() {
        if !d.exists() {
            let mut msg = String::new();
            write!(msg, "excluded path {} does not exist!", d.display()).unwrap();
            return Err(LoadError::new(msg));
        }
    }
    debug!("initial excluded code paths: {:?}", loaded_dirs);
    while settings.code_paths.len() > 0 {
        let dir = settings.code_paths.pop_front().unwrap(); // it has len, it better pop!
        if loaded_dirs.contains(&dir) {
            continue
        }
        debug!("Loading from code: {:?}", dir);
        match find_locs_dir(&dir, &mut loaded_dirs, &mut locs) {
            false => {},
            true => return Err(LoadError::new("encountered errors while finding locations".to_string())),
        }
    }
    Ok(locs)
}

/// attach the locations to the artifacts. Separated to allow for easy threading
pub fn attach_locs(artifacts: &mut Artifacts, locs: &HashMap<ArtName, Loc>) {
    for (lname, loc) in locs {
        let artifact = match artifacts.get_mut(lname) {
            Some(a) => a,
            None => continue,
        };
        artifact.loc = Some(loc.clone());
    }
}
