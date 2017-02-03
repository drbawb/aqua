use std::env;

use diesel::pg::PgConnection;
use iron::BeforeMiddleware;
use iron::prelude::*;
use iron::typemap;
use r2d2;
use r2d2_diesel::ConnectionManager;

type ConcretePool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbPool;
impl typemap::Key for DbPool { type Value = ConcretePool; }

/// Injects a thread-safe reference to a database connection pool into the extensions
/// for each request handled by a chain which includes this middleware.
pub struct DbMiddleware {
    pool: ConcretePool,
}

impl DbMiddleware {
    pub fn new() -> Self {
        // configure the database pool using environment
        let db_url  = env::var("DATABASE_URL").expect(".env missing key DATABASE_URL=postgres://<user>:<pw>@<host>/<db>");
        let config  = r2d2::Config::default();
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool    = r2d2::Pool::new(config, manager)
            .expect("could not setup db pool");


        DbMiddleware { pool: pool }
    }
}

impl BeforeMiddleware for DbMiddleware {
    /// Injects a connection from the database pool into this request's extensions
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let conn = self.pool.clone();
        req.extensions.insert::<DbPool>(conn); Ok(())
    }
}
