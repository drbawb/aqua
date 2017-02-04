use std::collections::HashMap;
use std::path::PathBuf;

use aqua_web::plug;
use mime_guess;

/// This middleware attempts to serve a static file from
/// the `./static` directory if it is available.
///
/// If the file is found an early response is generated; otherwise
/// the request is processed by the original handler.
pub struct TryFileMiddleware;

impl plug::Plug for TryFileMiddleware {
    fn call(&self, conn: &mut plug::Conn) {
        // NOTE: limits lexical borrow of `req`
        let try_path = {
            if conn.req().path().contains("./") || conn.req().path().contains("../") {
                panic!("hey man, that's just not cool... {}", conn.req().path());
            }

            PathBuf::from(format!("./static{}", conn.req().path()))
        };

        println!("checking path: {:?}", try_path);
        let file_exists = try_path.exists() && try_path.is_file();
        if file_exists {
            let mime_type = mime_guess::guess_mime_type(&try_path);
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), vec![mime_type.to_string()]);

            conn.send_file(200, try_path);
            conn.halt();
        }
    }
}
