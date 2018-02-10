# SPC-security
# General security features

In order to prevent mallicious or accidental user data corruption for
operations which edit user data, all operations which edit user
data **shall** meet the following criteria:

- all project files (whether editing or not) shall be subdirectories
    of the cwd repo.
- all edited files **shall** be subdirectories of settings.artifact_paths
- if a file is created, it must be separate from editing it.
- if any editing operation is not part of artifact (i.e. the web-ui), it
    shall require a valid password to perform any edits.

In order to accomplish this, a function shall be created which can
check a Project for discrepencies. This function shall be called
before any operation which edits user data is run.

This should at least help mitigate the risk that the user's entire
filesystem could be compromized (only the design folders can be

compromized)

## Risks
There is a *high chance* of a coding error which allows for editing
outside of the repo directory, which would cause an *avalance risk*
for security.

In essence, it could compromise all files which artifact has access to.

Lowest impact:
- user loosing design data in .toml files (i.e. missing text field, etc)

Highest impact:
- user loosing data *outside of the artifact repo*
- User's computer becomming compromised with malware installed by
    a mallicious user through artifact

There are two commands which this risk applies to:
- `art fmt` which edits user files to format them
- `art server` which allows online users to edit user files

Clearly `art server` exposes the user to the greatest risk as anyone
with access to the ip address + port of artifact could access their files
(unless some kind of password authentication is provided), whereas
art fmt would have to be created by an error in the application itself
(very possible, but less likely).

# TST-security
TODO: the main method of testing security shall be making sure:
- [[TST-security-bounds]]: you cannot edit files outside of the cwd
- TODO: you cannot edit files outside of the `artifact_paths` folders
- TODO: you cannot edit files over the web-ui without the correct
  credentials

# TST-security-bounds
There shall be two tests:
1. attempt (and fail) to load a project that has an artifact_path which is not 
    a subdirectory of that project.
2. attempt (and fail) to edit files which are not part of 
    settings.artifact_paths
