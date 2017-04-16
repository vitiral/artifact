extern crate diesel;

use dev_prefix::*;
use jsonrpc_core::{IoHandler, RpcMethodSync, Params, Error as RpcError};
use serde_json;

use diesel::prelude::*;
use api::establish_connection;
use api::types::*;
use api::utils;
use export::ArtifactData;

use super::ARTIFACTS;

/// the rpc initializer that implements the API spec
fn init_rpc_handler() -> IoHandler {
    // partof: #SPC-rpc-artifacts
    let mut handler = IoHandler::new();
    handler.add_method("GetArtifacts", GetArtifacts);
    // TODO: update is disabled until it is feature complete
    // (specifically security needs to be added)
    // handler.add_method("UpdateArtifacts", update::UpdateArtifacts);
    handler.add_method("GetTests", GetTests);
    handler.add_method("GetAllTestRuns", GetAllTestRuns);
    handler.add_method("GetRuns", GetRuns);
    handler.add_method("AddTestRun", AddTestRun);
    handler
}

lazy_static! {
    pub static ref RPC_HANDLER: IoHandler = init_rpc_handler();
}

/// `GetArtifacts` API Handler
struct GetArtifacts;
impl RpcMethodSync for GetArtifacts {
    fn call(&self, _: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("GetArtifacts called");
        let locked = ARTIFACTS.lock().unwrap();
        let artifacts: &Vec<ArtifactData> = locked.as_ref();
        Ok(serde_json::to_value(artifacts).expect("serde"))
    }
}

/// `GetTests` API Handler
struct GetTests;
impl RpcMethodSync for GetTests {
	fn call(&self, _: Params) -> result::Result<serde_json::Value, RpcError> {
	    use self::test_name::dsl::*;
        let connection = establish_connection();
        info!("GetTests called");

        let result = test_name.load::<TestName>(&connection)
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
		}
		else {
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
		
		let result = test_run::dsl::test_run.load::<TestRun>(&connection)
			.expect("Error loading test runs");
		
		Ok(serde_json::to_value(result).expect("serde"))
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
		let name_exists = test_name::table.filter(test_name::name.eq(&new_test_run.test_name))
			  .first::<TestName>(&connection); 
		
		if name_exists.is_err() {
			return Err(utils::invalid_params(&format!("Test name \'{}\' not in database. Please add using \'AddTest\' before continuing", new_test_run.test_name)));
		}
		
		//TODO: change variable names `a` and `c` to be more descriptive
		let a = diesel::insert(&new_test_run).into(test_run::table)
			.get_result::<TestRun>(&connection)
			.expect("Error adding new test run to database");
			
		let c = serde_json::to_value::<TestRun>(a).unwrap();
		
		Ok(c)
	}
}

