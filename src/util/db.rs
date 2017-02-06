use std::convert::From;
use std::error::Error;
use std::env;
use std::fmt;

use aqua_web::plug;
use aqua_web::result::Error as AquaError;
use diesel::result::Error as DieselError; 
use diesel::pg::PgConnection;
use r2d2::{self, Config, Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

/// The extension registry type of the database pool
pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;
pub type Result<T> = ::std::result::Result<T, DatabaseError>;

/// Injects a thread-safe reference to a database connection pool into the extensions
/// for each request handled by a chain which includes this middleware.
pub struct DbMiddleware { pool: DbPool }

pub fn fetch_conn(conn: &plug::Conn) -> Result<DbConn> {
    let pool = conn.find::<DbPool>()?;
    Ok(pool.get()?)
}

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

#[derive(Debug)]
pub enum DatabaseError {
    FrameworkError(AquaError),
    PoolTimeout(r2d2::GetTimeout),
    QueryError(DieselError),
}

impl Error for DatabaseError {
    fn description(&self) -> &str {
        match *self {
            DatabaseError::FrameworkError(ref err) => err.description(),
            DatabaseError::PoolTimeout(ref err)    => err.description(),
            DatabaseError::QueryError(ref err)     => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            DatabaseError::FrameworkError(ref err) => Some(err),
            DatabaseError::PoolTimeout(ref err)    => Some(err),
            DatabaseError::QueryError(ref err)     => Some(err),
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DatabaseError::FrameworkError(ref err) => err.fmt(f), 
            DatabaseError::PoolTimeout(ref err)    => err.fmt(f),
            DatabaseError::QueryError(ref err)     => err.fmt(f),
        }
    }
}

impl From<AquaError> for DatabaseError {
    fn from(err: AquaError) -> Self { DatabaseError::FrameworkError(err) }
}

impl From<DieselError> for DatabaseError {
    fn from(err: DieselError) -> Self { DatabaseError::QueryError(err) }
}

impl From<r2d2::GetTimeout> for DatabaseError {
    fn from(err: r2d2::GetTimeout) -> Self { DatabaseError::PoolTimeout(err) }
}
