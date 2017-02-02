use iron::BeforeMiddleware;
use iron::prelude::*;
use iron::typemap;
use models::ConcretePool;

pub struct DbPool;
impl typemap::Key for DbPool { type Value = ConcretePool; }

pub struct DbMiddleware {
    pool: ConcretePool,
}

impl DbMiddleware {
    pub fn new(db_pool: ConcretePool) -> Self {
        DbMiddleware { pool: db_pool }
    }
}

impl BeforeMiddleware for DbMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<DbPool>(self.pool.clone());
        Ok(())
    }
}
