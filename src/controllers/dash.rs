use controllers::prelude::*;
use models;
use views;

use aqua_web::plug;

#[derive(Serialize)]
struct Wat {
    derp: String,
}

/// Does the thing, wins the points ...
pub fn index(conn: &mut plug::Conn) {
    // db lulz
    let entries = ::models::queries::all_entries(conn.req());
    println!("got entries: {:?}", entries);

    // render template
    let data = Wat { derp: format!("entry => {:?}", entries) };
    let view = views::render_into(conn.req(), "layouts/main", "dash/index", &data);

    conn.send_resp(200, &view);
}

pub fn submit(conn: &mut plug::Conn) {
    error!("content_len: {:?}", conn.req().content_length());
    let mut buf = vec![];
    let wat = conn.req_mut().body().read_to_end(&mut buf).expect("oh my god");
    error!("actual size: {:?}", wat);

    conn.send_resp(200, "<h2>wat</h2>");
}

// TODO: (unwrap) trap file upload errors
// pub fn submit(req: &mut Request) -> IronResult<Response> {
//     use std::fs::File;
//     use std::io::Read;
// 
//     use crypto::digest::Digest;
//     use crypto::sha2::Sha256;
//     use params::{Params, Value};
// 
//     let form = req.get_ref::<Params>().unwrap();
//     if let Some(&Value::File(ref upload)) = form.get("upload") {
//         if !upload.path.exists() || !upload.path.is_file() {
//             return Ok(Response::with((status::BadRequest, "that file sux")));
//         }
// 
//         println!("file was pretty coo, gonna hash it");
//         let mut buf = vec![];
//         let hash = File::open(&upload.path)
//                         .and_then(|mut file| { file.read_to_end(&mut buf) })
//                         .map(|size| {
//        
//             println!("read {} bytes", size);
//             let mut digest = Sha256::new();
//             digest.input(&mut buf);
//             digest.result_str()
//         });
// 
//         match hash {
//             Ok(digest) => println!("got digest: {}", digest),
//             Err(msg) => println!("err reading file: {}", msg),
//         };
// 
// 
//         Ok(Response::with((status::Ok, "")))
//     } else {
//         Ok(Response::with((status::BadRequest, "that aint no file")))
//     }
// }
