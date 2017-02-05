use mw::route::Route;
use plug::{Conn, Plug};

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc,RwLock};

use conduit::Method;

// #[cfg(test)] use test::{black_box, Bencher};

type RouteMap = HashMap<Method, Vec<Route>>;

/// A `Router` is a middleware which attempts to match an HTTP request's
/// method and path to a corresponding handler function.
///
/// This is a simple implementation which simply attempts routes in the
/// order they were added to the router. Routes are grouped by HTTP method,
/// and as such matching a route takes at worst `O(n)` time where `n` is the
/// number of routes for the request method.
///
pub struct Router {
	routes: Arc<RwLock<RouteMap>>,
}

impl Router {
	pub fn new() -> Router {
		Router { routes: Arc::new(RwLock::new(HashMap::new())) }
	}

	/// Attaches a handler to a given route [regexp].
	pub fn add_route<P: Plug>(&mut self,  method:  Method, pattern: &str, handler: P) {
	
		// add it to our method-routes.
		let mut routes = self.routes.write()
            .expect("could not lock routing table for entry");
		let route = Route::new(pattern, handler);
		let mut route_list = match routes.entry(method) {
			Entry::Vacant(entry)   => entry.insert(Vec::new()),
			Entry::Occupied(entry) => entry.into_mut(),
		};

		route_list.push(route);
	}

    // TODO: macro to impl http verbs
    pub fn get<P: Plug>(mut self, pattern: &str, handler: P) -> Self {
        self.add_route(Method::Get, pattern, handler);
        self
    }

    pub fn post<P: Plug>(mut self, pattern: &str, handler: P) -> Self {
        self.add_route(Method::Post, pattern, handler);
        self
    }
}

impl Plug for Router {
    fn call(&self, conn: &mut Conn) {
 		let routes = self.routes.read().unwrap();
 		let handler = routes.get(&conn.req_mut().method()).and_then(|routes| {
 			println!("method found...");
 
 			routes.iter().find(|route| {
 				println!("checking route... {}", conn.req().path());
 				route.matches(&conn.req().path()[..])
 			})
 		});

        match handler {
            Some(route) => route.invoke_handler(conn),
            None => conn.send_resp(404, "router error: route not found"),
        }
    }
}

// #[cfg(test)]
// pub fn foo_handler(req: &Request, env: &mut Env) -> Result<String,String> {
// 	black_box(req);
// 	Err("ok".to_string())
// }
// 
// 
// #[bench]
// fn bench_clone_router_100(b: &mut Bencher) {
// 	let mut router = Router::new();
// 	for _ in (0..100) {
// 		router.add_route(Method::Put,
// 		                 "/foo/{bar}/baz",
// 		                 foo_handler);
// 	}
// 
// 	b.iter(|| { black_box(router.clone()); });
// }
