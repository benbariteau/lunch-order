extern crate iron;

use iron::{Request, Response, IronResult, Chain, Iron};

fn hello_world(request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

fn main() {
    let mut chain = Chain::new(hello_world);
    Iron::new(chain).http("localhost:8080").unwrap();
}
