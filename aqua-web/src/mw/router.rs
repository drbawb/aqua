use mw::route::Route;
use mw::{Segment, Outcome};

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc,RwLock};

use conduit::{Method, Request};

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
	pub fn add_route<S: Segment>(&mut self,  method:  Method, pattern: &str, handler: S) 
    where S: Send+Sync+'static {
	
		// add it to our method-routes.
		let mut routes = self.routes.write().unwrap();
		let route = Route::new(pattern, handler);
		let mut route_list = match routes.entry(method) {
			Entry::Vacant(entry)   => entry.insert(Vec::new()),
			Entry::Occupied(entry) => entry.into_mut(),
		};

		route_list.push(route);
	}
}

impl Segment for Router {
    fn invoke(&self, req: &mut Request) -> Outcome {
 		let routes = self.routes.read().unwrap();
 		let handler = routes.get(&req.method()).and_then(|routes| {
 			println!("method found...");
 
 			routes.iter().find(|route| {
 				println!("checking route... {}", req.path());
 				route.matches(&req.path()[..])
 			})
 		});

        match handler {
            Some(route) => route.invoke_handler(req),
            None => Outcome::Halt(404, "router failed: not found.".to_string()),
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
