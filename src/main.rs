extern crate iron;
extern crate router;

use iron::{Request, Response, IronResult, Chain, Iron};
use router::Router;

fn hello_world(request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

fn main() {
    let mut router = Router::new();
    router.get("/", hello_world, "home");
    let mut chain = Chain::new(hello_world);
    Iron::new(chain).http("localhost:8080").unwrap();
}
