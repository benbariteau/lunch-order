extern crate iron;
extern crate router;

use askama::Template;
use iron::{Request, Response, IronResult, Chain, Iron, status};
use router::Router;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    restaurant_list: Vec<String>
}

fn index(request: &mut Request) -> IronResult<Response> {
    Ok(
        Response::with((
            status::Ok,
            IndexTemplate{restaurant_list: vec![]},
        ))
    )
}

fn main() {
    let mut router = Router::new();
    router.get("/", index, "home");
    let mut chain = Chain::new(router);
    Iron::new(chain).http("localhost:8080").unwrap();
}
