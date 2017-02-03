use controllers::prelude::*;
use views;

#[derive(Serialize)]
struct Wat {
    derp: String,
}

/// Does the thing, wins the points ...
pub fn index(req: &mut Request) -> IronResult<Response> {
    // db lulz
    let entries = ::models::queries::all_entries(req);
    println!("got entries: {:?}", entries);


    // render template
    let data = Wat { derp: format!("entry => {:?}", "oh no") };
    let view = views::render_into(req, "layouts/main", "dash/index", &data);
    Ok(Response::with((status::Ok, view))
                .set(mime!(Text/Html)))
}

// TODO: (unwrap) trap file upload errors
pub fn submit(req: &mut Request) -> IronResult<Response> {
    use std::fs::File;
    use std::io::Read;

    use crypto::digest::Digest;
    use crypto::sha2::Sha256;
    use params::{Params, Value};

    let form = req.get_ref::<Params>().unwrap();
    if let Some(&Value::File(ref upload)) = form.get("upload") {
        if !upload.path.exists() || !upload.path.is_file() {
            return Ok(Response::with((status::BadRequest, "that file sux")));
        }

        println!("file was pretty coo, gonna hash it");
        let mut buf = vec![];
        let hash = File::open(&upload.path)
                        .and_then(|mut file| { file.read_to_end(&mut buf) })
                        .map(|size| {
       
            println!("read {} bytes", size);
            let mut digest = Sha256::new();
            digest.input(&mut buf);
            digest.result_str()
        });

        match hash {
            Ok(digest) => println!("got digest: {}", digest),
            Err(msg) => println!("err reading file: {}", msg),
        };


        Ok(Response::with((status::Ok, "")))
    } else {
        Ok(Response::with((status::BadRequest, "that aint no file")))
    }
}
