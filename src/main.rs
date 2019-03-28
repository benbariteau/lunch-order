#[macro_use]
extern crate diesel;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate iron;

use askama::Template;
use diesel::{
    connection::Connection,
    deserialize::Queryable,
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
    Request,
    Response,
    status,
};
use logger::Logger;
use router::Router;
use serde::Deserialize;
use std::io::Read;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    restaurant_list: Vec<String>
}

#[derive(Template)]
#[template(path = "add.html")]
struct AddTemplate {}

#[derive(Queryable)]
struct Restaurant {
    id: i32,
    name: String,
    last_visit_date: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "restaurant"]
struct NewRestaurant {
    name: String,
    last_visit_date: String,
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

mod schema {
    table! {
        restaurant {
            id -> Integer,
            name -> VarChar,
            last_visit_date -> VarChar,
        }
    }
}
use self::schema::restaurant;

fn create_db_connection() -> ConnectionResult<SqliteConnection> {
    SqliteConnection::establish("lunch_order.db")
}

fn get_restaurants() -> errors::LunchOrderResult<Vec<Restaurant>> {
    let db_connection = create_db_connection()?;
    let restaurants: Vec<Restaurant> = schema::restaurant::table
        .order(schema::restaurant::last_visit_date.desc())
        .load(&db_connection)?;
    Ok(restaurants)
}

fn create_restaurant(restaurant: &NewRestaurant) -> errors::LunchOrderResult<()> {
    let db_connection = create_db_connection()?;
    diesel::insert_into(schema::restaurant::table)
        .values(restaurant)
        .execute(&db_connection)?;
    Ok(())
}

fn index(request: &mut Request) -> IronResult<Response> {
    let restaurant_names = itry!(get_restaurants())
        .into_iter()
        .map(|restaurant| restaurant.name)
        .collect();
    Ok(
        Response::with((
            status::Ok,
            IndexTemplate{restaurant_list: restaurant_names},
        ))
    )
}

fn add_form(request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((
        status::Ok,
        AddTemplate{},
    )))
}

fn add(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    itry!(request.body.read_to_string(&mut body));
    let new_restaurant: NewRestaurant = itry!(serde_urlencoded::from_str(&body));
    itry!(create_restaurant(&new_restaurant));
    Ok(Response::new())
}

fn main() {
    env_logger::init();

    let mut router = Router::new();
    router.get("/", index, "home");
    router.get("/add", add_form, "add_form");
    router.post("/add", add, "add");

    let mut chain = Chain::new(router);

    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("localhost:8080").unwrap();
}
