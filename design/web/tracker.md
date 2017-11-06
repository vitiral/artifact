# SPC-tracker
TODO: The tracker specification will be defined by the tracker team

# SPC-tracker-schema
The data that needs to be stored by the test tracking tool is:
 - test name (i.e. MyTest2)
 - artifacts it tests (i.e. TST-foo-bar)
 - date test occured
 - version that was tested
 - url/link to view test results
 - extra data to store

The schema is designed to be as simple as possible, holding only the data
which might be useful to external infrastructure and no more.

All versions use a standardized format of <major>.<minor>.<patch> with
an additional <build> parameter for distinguishing between different
development runs.

The version standardization is to make it easy to sort versions by their
various attributes.

```
Notes: 
    PK = primary-key
    P  = standard id

## TEST_NAME
      key | type                        | description
    ------+-----------------------------+-------------------------
     name | text                     PK | test name can be any string

## ARTIFACT_NAME
      key | type                        | description
----------+-----------------------------+-------------------------
     name | text ONLY -0-9A-Z_       PK | name array, reduced form of ArtName

## VERSION
      key | type                        | description
----------+-----------------------------+-------------------------
       id | id                       P  | version id
    major | text                        | "major" version str
    minor | text                        | "minor" version str
    patch | text                        | "patch" version str
    build | text                        | "build" str, typically a hash

## RUN
      key | type                        | description
----------+-----------------------------+-------------------------
       id | id                       P  | run id
  name id | name-id                     | test name
   passed | bool                        | true if passed, false if failed
artifacts | artifact-name-id[]          | artifact name-ids tested
     date | datetime                    | date the test was run
  version | version-id                  | version test was run
     link | text                        | url or path for more info on test
     data | u8[]                        | additional raw data (i.e. error report)

```
