# SPC-data-cache
The data cache has the following goals:
- save on memory
- reduce operating system calls
- minor savings on speed

The cached datatypes are:
- `Name`: `Arc<InternalName>`: Reference counted "artifact name", which is used as the key for artifacts as well as specifying parts/partof, code-locations, subnames, etc. Copies of these values can exist several times over, which is why it makes sense to reference count them.
- `PathAbs`: `Arc<PathBuf>`: A cache of absolute paths, with lookups based on the "query path". Looking up the abosolute path requires a call to [canonicalize()](https://doc.rust-lang.org/1.10.0/std/fs/fn.canonicalize.html), which requires a call to the operating system and is quite slow. Paths are used in artifact-definition, code-location, subcode-location and parsed-paths and are often replicated, therefore it makes sense to cache and reference count them.


## [[.name]]: cache of artifact names
- Has two sub-caches:
  - `HashSet<Arc<String::to_uppercase>>`: reference counted set of all keys to reduce overhead
  - `HashMap<String, Arc<InternalName>>`: map of raw-name to instantiated name.
- The `NameCache` object has the following methods:
  - `get(name: &str) -> Result<Name>`: return the cached Name object.
- Names are looked up by their raw representation.
- If they do not exist then they are validated and inserted.
- This saves on validating identical names if they already exist

## [[.path]]: cache of paths.
- Has two sub-caches:
  - `HashSet<PathAbs>` reference counted set of all known paths.
  - `HashMap<PathBuf, PathAbs>`: this contains a memoized reference of all
    input paths and their `canonicalize()` path (which requires an OS call).
- The `CachedPaths` object has the following methods:
  - `get(path: &Path) -> Result<PathRc>`: return the canonicalized
    path in the fastest way possible.
