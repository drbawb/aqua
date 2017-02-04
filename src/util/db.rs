use std::env;

use aqua_web::mw::{Outcome, Segment, Wrapper};
use conduit::Request;
use diesel::pg::PgConnection;
use r2d2::{Config, Pool};
use r2d2_diesel::ConnectionManager;


/// This can be used to add a database pool (configured by .env) to 
/// a middleware chain. This will inject the database pool into the
/// request extensions and then forward the request to the next handler
/// in the chain ...
///
pub struct DatabaseMiddleware;

impl Wrapper for DatabaseMiddleware {
    fn around(self, handler: Box<Segment>) -> Box<Segment> {
        Box::new(DbHandler::new(handler))
    }
}

/// The extension registry type of the database pool
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Injects a thread-safe reference to a database connection pool into the extensions
/// for each request handled by a chain which includes this middleware.
struct DbHandler {
    next: Box<Segment>,
    pool: DbPool,
}

impl DbHandler {
    pub fn new(next: Box<Segment>) -> Self {
        // configure the database pool using environment
        let db_url  = env::var("DATABASE_URL").expect(".env missing key DATABASE_URL=postgres://<user>:<pw>@<host>/<db>");
        let config  = Config::default();
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool    = Pool::new(config, manager)
            .expect("could not setup db pool");


        DbHandler { next: next, pool: pool }
    }
}

impl Segment for DbHandler {
    /// Injects a connection from the database pool into this request's extensions
    fn invoke(&self, req: &mut Request) -> Outcome {
        let conn = self.pool.clone();
        req.mut_extensions().insert::<DbPool>(conn);
        self.next.invoke(req)
    }
}
