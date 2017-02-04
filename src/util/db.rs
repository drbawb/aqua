use std::env;

use aqua_web::plug;
use diesel::pg::PgConnection;
use r2d2::{Config, Pool};
use r2d2_diesel::ConnectionManager;


/// The extension registry type of the database pool
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Injects a thread-safe reference to a database connection pool into the extensions
/// for each request handled by a chain which includes this middleware.
pub struct DbMiddleware { pool: DbPool }

impl DbMiddleware {
    pub fn new() -> Self {
        // configure the database pool using environment
        let db_url  = env::var("DATABASE_URL").expect(".env missing key DATABASE_URL=postgres://<user>:<pw>@<host>/<db>");
        let config  = Config::default();
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool    = Pool::new(config, manager)
            .expect("could not setup db pool");


        DbMiddleware { pool: pool }
    }
}

impl plug::Plug for DbMiddleware {
    /// Injects a connection from the database pool into this request's extensions
    fn call(&self, conn: &mut plug::Conn) {
        let db_conn = self.pool.clone();
        conn.req_mut().mut_extensions().insert::<DbPool>(db_conn);
    }
}
