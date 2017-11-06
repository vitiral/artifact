# SPC-rpc
The rpc server shall be hosted at `<url>/json-rpc` and shall adhere to the
[JSON-RPC 2.0][1] specification.

It has the following components:
- [[SPC-rpc-artifacts]]: calls for create/read/update/delete artifacts
- [[SPC-rpc-fs]]: calls for create/read/update/delete files and folders

[1]: http://www.jsonrpc.org/specification

# SPC-rpc-artifacts
## ReadProject
Get the current project state

Note: this is NOT ReadArtifacts because there is more information that
can change and it is important to update the web-app with the full
information every time (and not let anything get out of sync).

### params:

No parameters (the entire project is always returned)

### result:
- artifacts `list[Artifact]`: List containing all artifacts
- files: `list[String]`: list of all file paths. Note that some files
  may be empty.
    
### error:

This command should not return any errors.

Notes:
All artifacts are guaranteed to have a unique id and their id will
never change (until they are deleted). They also have a revision,
which is incremented every time there is a user change.

## UpdateArtifacts
update (edit) existing artifacts

### params:
- artifacts: list[Artifact]: artifact objects with updated data.
  The updated artifacts will be returned with their `revision` field
  increased by one.

### result:

Same result as `ReadProject`. The fully updated project is
returned on success to inform the webapp of the changes immediately.

### error:

message:
- xIdsNotFound error if any ids don't exist
- xFilesNotFound error if an artifact contains a non-registered file 
  (files have to be created through a separate API call
- xInvalidName for invalid artifact name
- xNameExists error if any of the artifact names already exist
- xInvalidPartof for invalid partof name
- xMultipleErrors if multiple types of errors occured. See `data`.

data:
The data contains a list of all error messages and additional data.

## CreateArtifacts
Create new artifacts.

### params:
- `artifacts list[Artifact]`: new artifact objects that have their
  id set to 0

### result:

Same result as `ReadProject`. The fully updated project is
returned on success to inform the webapp of the changes immediately.

### error:
The errors are identical to `UpdateArtifacts` except for `xIdsNotFound`
is replaced with `xInvalidId` if the id does not equal 0.

## DeleteArtifacts
Delete artifacts, removing their id.

### params
- `ids list[int]`: list of artifact ids to delete

### result
Same result as `ReadProject`. The fully updated project is
returned on success to inform the webapp of the changes immediately.

### error
- xIdsNotFound error if any ids don't exist

# SPC-rpc-fs
filesystem related rpc.

Note that existing files are obtained through `ReadProject` defined
in [[SPC-rpc-artifacts]]

# CreateFiles
params:
- paths: list[string]: relative paths to create

error:
- xFolderNotFound if any folders in the path don't exist
- xPathExists if the path is used
- xInvalidExt if the file doesn't end in `.toml`


# CreateFolders
params:
- paths: list[string]: relative paths to create folders

error:
- xFolderNotFound if any folders don't exist
- xPathExists if the path is used
- xPathOutOfBounds if the path would not be found
    within the `artifact_paths` variable


# TODO:
- DeleteFiles
- DeleteFolders
