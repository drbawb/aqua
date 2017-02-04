use aqua_web::mw::{Outcome, Segment, Wrapper};
use conduit::Request;
use time::precise_time_ns;

pub struct RequestTimer;

impl Wrapper for RequestTimer {
    fn around(self, handler: Box<Segment>) -> Box<Segment> {
        Box::new(RequestTimeHandler(handler))
    }
}

struct RequestTimeHandler(Box<Segment>);

impl Segment for RequestTimeHandler {
    fn invoke(&self, req: &mut Request) -> Outcome {
        println!("--- Request start ---");
        let start = precise_time_ns();
        let outcome = self.0.invoke(req);
        let delta = precise_time_ns() - start;
        println!("Request took: {}ms\n", (delta as f64) / 1000000.0);
        outcome
    }
}
