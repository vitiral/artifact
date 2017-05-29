use dev_prefix::*;
use jsonrpc_core::{IoHandler, RpcMethodSync, Params, Error as RpcError};
use serde_json;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use api::utils;
use dotenv::dotenv;

use tracker::types::*;

pub fn init_rpc_handler_tracker(handler: &mut IoHandler) {
    handler.add_method("GetTests", GetTests);
    handler.add_method("GetAllTestRuns", GetAllTestRuns);
    handler.add_method("GetRuns", GetRuns);
    handler.add_method("AddTestRun", AddTestRun);
    handler.add_method("AddVersion", AddVersion);
    handler.add_method("AddTest", AddTest);
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connection to {}", database_url))
}

/// `GetTests` API Handler
struct GetTests;
impl RpcMethodSync for GetTests {
    fn call(&self, _: Params) -> result::Result<serde_json::Value, RpcError> {
        use self::test_name::dsl::*;
        let connection = establish_connection();
        info!("GetTests called");

        let result = test_name
            .load::<TestName>(&connection)
            .expect("Error loading test names");

        Ok(serde_json::to_value(result).expect("serde"))
    }
}

/// `GetRuns` API Handler
struct GetRuns;
impl RpcMethodSync for GetRuns {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("GetRuns called");
        let connection = establish_connection();


        let val = serde_json::to_value(params).unwrap();
        if let Ok(test_run_search) = serde_json::from_value::<TestRunSearch>(val) {

            let mut version_ids: Vec<i32> = Vec::new();

            let mut query = test_run::table.into_boxed();

            if test_run_search.versions.is_some() {
                for v in test_run_search.versions.unwrap() {
                    // build the query for table `version`
                    let mut v_query = version::table.select(version::id).into_boxed();
                    v_query = v_query.filter(version::major.eq(v.major));

                    if let Some(minor) = v.minor {
                        v_query = v_query.filter(version::minor.eq(minor));
                    }
                    if let Some(patch) = v.patch {
                        v_query = v_query.filter(version::patch.eq(patch));
                    }
                    if let Some(build) = v.build {
                        v_query = v_query.filter(version::build.eq(build));
                    }

                    for k in v_query.load(&connection).unwrap() {
                        version_ids.push(k);
                    }

                }
                query = query.filter(test_run::version_id.eq_any(version_ids));
            }

            //let mut finalresult: Vec<TestRun> = Vec::new();

            if let Some(min_epoch) = test_run_search.min_epoch {
                query = query.filter(test_run::epoch.ge(min_epoch));
            }
            if let Some(max_epoch) = test_run_search.max_epoch {
                query = query.filter(test_run::epoch.le(max_epoch));
            }

            let finalresult = query.load::<TestRun>(&connection).unwrap();


            Ok(serde_json::to_value(finalresult).unwrap())
        } else {
            Err(utils::invalid_params("Missing parameters"))
        }


    }
}

/// `GetAllTestRuns` API handler
struct GetAllTestRuns;
impl RpcMethodSync for GetAllTestRuns {
    fn call(&self, _: Params) -> result::Result<serde_json::Value, RpcError> {
        let connection = establish_connection();
        info!("GetAllTestRuns called");

        let result = test_run::dsl::test_run
            .load::<TestRun>(&connection)
            .expect("Error loading test runs");

        Ok(serde_json::to_value(result).expect("serde"))
    }
}

/// `AddVersion` API Handler
struct AddVersion;
impl RpcMethodSync for AddVersion {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("AddVersion called");
        let connection = establish_connection();

        let val = serde_json::to_value(params).unwrap();
        if let Ok(new_version) = serde_json::from_value::<NewVersion>(val) {
            let insert_result = diesel::insert(&new_version)
                .into(version::table)
                .get_result::<Version>(&connection)
                .expect("Error adding new version to database");

            Ok(serde_json::to_value::<Version>(insert_result).unwrap())
        } else {
            Err(utils::invalid_params("Cannot parse parameters"))
        }
    }
}

/// `AddTestRun` API Handler
struct AddTestRun;
impl RpcMethodSync for AddTestRun {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("AddTestRun called");
        let connection = establish_connection();

        let val = serde_json::to_value(params).unwrap();
        let new_test_run: NewTestRun = serde_json::from_value(val).unwrap();
        info!("{:?}", new_test_run);

        //check test_name table for existance of test_name
        if test_name::table
               .filter(test_name::name.eq(&new_test_run.test_name))
               .first::<TestName>(&connection)
               .is_err() {
            let msg = format!("Test name \'{}\' not in database. Please \
                add using \'AddTest\' before continuing",
                              new_test_run.test_name);
            return Err(utils::invalid_params(&msg));
        }

        // check if version_id is valid
        if version::table
               .filter(version::id.eq(&new_test_run.version_id))
               .first::<Version>(&connection)
               .is_err() {
            let msg = format!("Version id \'{}\' not in database. \
                Please add using \'AddVersion\' before continuing",
                              new_test_run.version_id);
            return Err(utils::invalid_params(&msg));
        }

        // check if artifacts are in database
        for artifact in &new_test_run.artifacts {
            //let art_name = ArtifactName { name: artifact.clone() };
            if artifact_name::table
                   .filter(artifact_name::name.eq(artifact))
                   .first::<ArtifactName>(&connection)
                   .is_err() {
                let msg = format!("Artifact \'{}\' not in database. \
                    Please add using \'AddArtifact\' before continuing",
                                  artifact);
                return Err(utils::invalid_params(&msg));
            }
        }

        //TODO: change variable names `a` and `c` to be more descriptive
        let a = diesel::insert(&new_test_run)
            .into(test_run::table)
            .get_result::<TestRun>(&connection)
            .expect("Error adding new test run to database");

        let c = serde_json::to_value::<TestRun>(a).unwrap();

        Ok(c)
    }
}

/// `AddTests` API Handler
struct AddTest;
impl RpcMethodSync for AddTest {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("AddTest called");
        use self::test_name::dsl::*;
        use self::test_name;
        use serde_json::Value as SerdeValue;


        let connection = establish_connection();

        info!("{:?}", params);


        //Checking if the test name is already in the test_name table
        let serialized_params = serde_json::to_value(params).unwrap();
        let mut newTest: TestName = serde_json::from_value(serialized_params).unwrap();
        let results = test_name
            .filter(test_name::name.eq(&newTest.name))
            .load::<TestName>(&connection)
            .expect("Error loading test name table");
        for TestName in results {
            let d = serde_json::to_value("Test is already added").unwrap();
            println!("{:?}", d);
            return Ok(d);
        }
        let a = diesel::insert(&newTest)
            .into(test_name::table)
            .get_result::<TestName>(&connection)
            .expect("Error adding new test to database");


        let c = serde_json::to_value::<TestName>(a).unwrap();

        println!("{:?}", c);

        return Ok(c);


    }
}
