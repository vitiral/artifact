# specifications for the REST api server

[SPC-rest]
partof = "REQ-2-rest"
text = '''
The REST API server **shall** be started when the web-ui is started.
It shall be a json-rpc compliant REST API server with a simple CRUD
interface.

The supported comands **shall** be:
 - GetRuns: get runs given search criteria
 - AddRuns: create test run instances
 - ModifyRun: modify a run id
 - DeleteRuns: delete a list of run-ids

 - GetVersions: get runs given search criteria
 - AddVersions: create test run instances
 - ModifyVersion: modify a run id
 - DeleteVersions: delete a list of run-ids

There **will** be 3 levels of authentication available:
 - viewer: can only Get*
 - tester: can do viewer + AddRuns + AddVersions
 - admin: can do anything

in order to submit a test run, the version must already be added.
'''
