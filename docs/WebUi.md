# Artifact Web UI

For an example web ui, see [artifact's own design docs][2]

The artifact Web UI is intended to be simple and intuiative, however
it can be helpful to have some context.

For more information about artifact, see it's [homepage][1]

[1]: https://github.com/vitiral/artifact
[2]: http://vitiral.github.io/artifact/#artifacts/req-1

## Artifact Color

An "artifact" is something with a name like "ART-name" where "ART" is
one of:
- `REQ`: a requirement
- `RSK`: a risk
- `SPC`: a design specification
- `TST`: a test

Artifacts can take on different colors representing:
- green: artifact is completed and tested
- blue: artifact is completed but not tested (or partially complete and tested)
- yellow: artifact is somewhat complete/tested
- red: artifact is not very complete or tested

## List View

The base view is the List View. It contains a column selector at the top
left, a search bar at the top right, and a list of the searched artifacts
below.

### Column Selector
The column selector lets you select columns you wish to view. Displayed
columns will be black and hidden columns will be grey.

### Search Bar
The search bar allows you to search in the `name`, `parts`, `partof` and/or
`text` fields for the value in the search bar. By default it searches for
(case insensitive) text, but you can also enable regex searching with the
button to the right of the search bar.
