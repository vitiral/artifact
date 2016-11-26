fn parse_id(req: &Request) -> Result<u64, String> {
    match req.param("id").expect("invalid url got through").parse() {
        Ok(id) => Ok(id),
        Err(err) => {
            println!("- got invalid id");
            Err(format!("invalid id: {}", err))
        },
    }
}



fn get_artifact<'a>(artifacts: &'a mut Vec<Artifact>, id: u64) -> Result<&'a mut Artifact, String> {
    match artifacts.iter_mut().filter(|p| p.id == id).next() {
        Some(a) => Ok(a),
        None => {
            println!("- id not found: {}", id);
            Err(format!("Artifact {} not found", id))
        },
    }
}

fn get_artifact_id<'a> (req: &mut Request, mut res: Response<'a>) 
        -> MiddlewareResult<'a> 
{
    setup_headers(&mut res);
    let id = match parse_id(req) {
        Ok(id) => id,
        Err(e) => {
            res.set(StatusCode::BadRequest);
            return res.send(e);
        }
    };
    let mut locked = ARTIFACTS.lock().unwrap();
    let artifact = match get_artifact(locked.as_mut(), id) {
        Ok(a) => a,
        Err(e) => {
            res.set(StatusCode::NotFound);
            return res.send(e);
        },
    };
    let data = json::as_pretty_json(artifact);
    let str_data = format!("{}", data);
    println!("* GET /artifacts/{} -> {}", id, str_data);
    config_json_res(&mut res);
    res.send(str_data)
}

fn put_artifact_id<'a> (req: &mut Request, mut res: Response<'a>) 
        -> MiddlewareResult<'a> 
{
    println!("* PUT artifacts/:id start");
    setup_headers(&mut res);
    let id = match parse_id(req) {
        Ok(id) => id,
        Err(e) => {
            res.set(StatusCode::NotFound);
            return res.send(e);
        },
    };
    let mut locked = ARTIFACTS.lock().unwrap();
    let artifact = match get_artifact(locked.as_mut(), id) {
        Ok(a) => a,
        Err(e) => {
            res.set(StatusCode::NotFound);
            return res.send(e);
        },
    };
    let new = match req.json_as::<Artifact>() {
        Ok(a) => a,
        Err(e) => {
            res.set(StatusCode::BadRequest);
            return res.send(format!("{}", e));
        },
    };
    if new.id != id {
        res.set(StatusCode::BadRequest);
        return res.send("cannot change artifact's id");
    }
    if new == *artifact {
        res.set(StatusCode::NotModified);
        return res.send("not modified");
    }
    *artifact = new;
    let data = json::as_pretty_json(artifact);
    let str_data = format!("{}", data);
    println!("* PUT /artifacts/{} success", id);
    config_json_res(&mut res);
    res.send(str_data)
}
