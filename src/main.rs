#[macro_use]
extern crate diesel;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate iron;

use askama::Template;
use chrono::{
    DateTime,
    offset::{
        Local,
        Utc,
    },
};
use diesel::{
    connection::Connection,
    ExpressionMethods,
    QueryDsl,
    RunQueryDsl,
    result::ConnectionResult,
    sqlite::SqliteConnection,
};
use env_logger;
use iron::{
    Chain,
    Iron,
    IronResult,
    modifiers::RedirectRaw,
    Request,
    Response,
    status,
};
use logger::Logger;
use mount::Mount;
use router::Router;
use serde::Deserialize;
use std::io::Read;
use std::path::Path;

mod model;
mod schema;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    restaurant_list: Vec<RestaurantPresenter>
}

#[derive(Deserialize)]
struct RestaurantForm {
    name: String,
}

struct RestaurantPresenter {
    restaurant_model: model::Restaurant,
    level: u8,
    time_string: String,
}

mod errors {
    error_chain! {
        types {
            LunchOrderError,
            LunchOrderErrorKind,
            LunchOrderResultExt,
            LunchOrderResult;
        }
        foreign_links {
            DbConnectionError(::diesel::result::ConnectionError);
            DbError(::diesel::result::Error);
        }
    }
}

fn create_db_connection() -> ConnectionResult<SqliteConnection> {
    SqliteConnection::establish("lunch_order.db")
}

fn get_restaurants() -> errors::LunchOrderResult<Vec<model::Restaurant>> {
    let db_connection = create_db_connection()?;
    let restaurants: Vec<model::Restaurant> = schema::restaurant::table
        .order(schema::restaurant::last_visit_time.asc())
        .load(&db_connection)?;
    Ok(restaurants)
}

fn create_restaurant(restaurant: &model::NewRestaurant) -> errors::LunchOrderResult<()> {
    let db_connection = create_db_connection()?;
    diesel::insert_into(schema::restaurant::table)
        .values(restaurant)
        .execute(&db_connection)?;
    Ok(())
}

fn update_restaurant(id: i32, visit_time: String) -> errors::LunchOrderResult<()> {
    let db_connection = create_db_connection()?;
    diesel::update(schema::restaurant::table.find(id))
        .set(schema::restaurant::last_visit_time.eq(visit_time))
        .execute(&db_connection)?;
    Ok(())
}

fn index(_request: &mut Request) -> IronResult<Response> {
    let restaurant_list = itry!(get_restaurants());
    let coefficient = (restaurant_list.len() as f64) / 5.0;
    let restaurant_presenters = restaurant_list
        .into_iter()
        .enumerate()
        .map(
            |(i, restaurant)| {
                let level = if i < coefficient as usize {
                    0
                } else if i < (2.0*coefficient) as usize {
                    1
                } else if i < (3.0*coefficient) as usize {
                    2
                } else if i < (4.0*coefficient) as usize {
                    3
                } else {
                    4
                };
                let last_visit_time: DateTime<Utc> = restaurant.last_visit_time.parse().unwrap();
                RestaurantPresenter{
                    restaurant_model: restaurant,
                    level: level,
                    time_string: format!("visited {} days ago", (Utc::now() - last_visit_time).num_days()),
                }
            }
        )
        .collect();
    Ok(
        Response::with((
            status::Ok,
            IndexTemplate{restaurant_list: restaurant_presenters},
        ))
    )
}

fn add(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    itry!(request.body.read_to_string(&mut body));
    let restaurant_form: RestaurantForm = itry!(serde_urlencoded::from_str(&body));
    let new_restaurant = model::NewRestaurant{
        name: restaurant_form.name,
        last_visit_time: Local::now().to_rfc3339(),
    };
    itry!(create_restaurant(&new_restaurant));
    Ok(Response::with((
        status::SeeOther,
        RedirectRaw("/".to_string()),
    )))
}

fn visit(request: &mut Request) -> IronResult<Response> {
    let id: i32 = itry!(request.extensions.get::<Router>().unwrap().find("id").unwrap().parse());
    itry!(update_restaurant(id, Local::now().to_rfc3339()));
    Ok(Response::with((
        status::SeeOther,
        RedirectRaw("/".to_string()),
    )))
}

fn main() {
    env_logger::init();

    let mut router = Router::new();
    router.get("/", index, "home");
    router.post("/add", add, "add");
    router.post("/visit/:id", visit, "visit");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static/", staticfile::Static::new(Path::new("static/")));

    let mut chain = Chain::new(mount);

    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("localhost:8080").unwrap();
}
