
[REQ-db]
partof = "REQ-2-rest"
text = '''
there **will** be a database which serves as a backend to the
API server. This database **will** be a relational model (SQL)
database which will allow the REST API to function and provide
the data for the web and cmdline ui's to view data.
'''

[SPC-db]
[SPC-db-schema]
text = '''
The data that needs to be stored by the test tracking tool is:
 - test name (i.e. TST-foo-bar)
 - date test occured
 - version that was tested
 - url/link to view test results
 - extra data to store

# Schemas

The schema is designed to be as simple as possible, holding only the data
which might be useful to external infrastructure and no more.

All versions use a standardized format of <major>.<minor>.<patch> with
an additional <build> parameter for distinguishing between different
development runs.

The version standardization is to make it easy to sort versions by their
various attributes.

```
## NAMES
      key | type                        | description
----------+-----------------------------+-------------------------
       id | <id>                        | name id
     name | [<str>] ONLY 0-9A-Z_        | name array, reduced form of ArtName
      raw | <str>   ONLY -0-9A-Za-z_    | readable unicode name, must convert through
                                        |   ArtName rules to `name`

## RUNS
      key | type                        | description
----------+-----------------------------+-------------------------
       id | <id>                        | run id
 name_ids | [<name id>]                 | list of name-ids that this test tests
     date | <datetime>                  | date the test was run
  version | <version id>                | link to version test was run
     link | <str>                       | url or path to more info on test
     data | [<u8>]                      | additional raw data (i.e. error report)

## VERSIONS
      key | type                        | description
 ---------+-----------------------------+-------------------------
       id | <id>                        | version id
    major | <str>                       | "major" version number
    minor | <str>                       | "minor" version number
    patch | <str>                       | "patch" version number
    build | <str>                       | "build" id, typically a hash
```
'''
