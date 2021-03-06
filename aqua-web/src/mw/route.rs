use std::collections::HashMap;

use mw::regexp::Expression;
use plug::{Conn, Plug};

pub type MatchContext = HashMap<String,String>;
pub type Handler = Box<Plug>;

/// A route is a combination of a compiled matcher along with an
/// invokable handler.
///
/// If the route matches the corresponding entries will be stored
/// in the request's extension under the `MatchContext` type, and
/// the handler will then be called.
///
pub struct Route {
	matcher: Expression,
	handler: Handler,
}

impl Route {
    /// Create a route using a template string and a stack `Segment`.
    /// Please see the documentation for `Expression` to learn more
    /// about the formatting of the template string.
	pub fn new<P: Plug>(template: &str, handler: P) -> Route {
		Route {
			matcher: Expression::from_template(template).unwrap(),
			handler: Box::new(handler),
		}
	}

	// TODO: return Option<MatchMetadata> or something along those lines ...
	/// Determine if this route matches the current path
	pub fn matches(&self, path: &str) -> bool {
		self.matcher.is_match(path)
	}

    /// Apply this route to a path and retrieve a match context if possible.
	pub fn get_context(&self, path: &str) -> MatchContext {
		self.matcher.map_path(path)
	}

	/// Stores parameters from the matched route into the request's environment.
	/// Then the route's associated function pointer is invoked and given
	/// a chance to modify the response.
	///
	/// Any error's raised by the route will be stored in the `Response.err` field.
	pub fn invoke_handler(&self, conn: &mut Conn) {
        // NOTE: binding is here just for lexical scope (to borrow req)
        let context = { self.get_context(conn.req().path()) };
        conn.req_mut().mut_extensions().insert::<MatchContext>(context);
        (*self.handler).call(conn)
	}
}

#[cfg(test)]
mod test {
    use super::*;
    use conduit::Request;
    use plug::{Conn, Plug};

    struct MockHandler;
    impl Plug for MockHandler {
        fn call(&self, conn: &mut Conn) {}
    }

 	#[test]
 	fn test_route_matches() {
 		let route = Route::new("/foo/{bar}/{baz}", MockHandler);
 		assert!(route.matches("/foo/hello/test"));
 	}

 	#[test]
 	fn test_route_matches_not_greedy() {
 		let route = Route::new("/foo/{bar}/{baz}/quux", MockHandler);
 		assert!(!route.matches("/foo/hello/test"));
 	}

 	#[test]
 	fn test_route_params_nonempty() {
 		let route = Route::new("/foo/{bar}/{baz}", MockHandler);
 		let params = route.get_context("/foo/hello/test");
 		assert!(params.get("bar").is_some());
 		assert!(params.get("baz").is_some());
 	}

 	#[test]
 	fn test_route_params_empty() {
 		let route = Route::new("/foo/{bar}/{baz}", MockHandler);
 		let params = route.get_context("/foo//test");
 		assert!(params.get("bar").is_some());
 		assert!(params.get("baz").is_some());
 	}

    // TODO: nirvash used to have a test of invoking the handler
    //       this would require a mock `conduit::Request` object w/ working
    //       extension storage. 
}
