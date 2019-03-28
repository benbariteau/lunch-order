#[macro_use]
extern crate diesel;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate iron;

use askama::Template;
use diesel::connection::Connection;
use diesel::deserialize::Queryable;
use diesel::{
    ExpressionMethods,
    QueryDsl,
    RunQueryDsl,
};
use diesel::result::ConnectionResult;
use diesel::sqlite::SqliteConnection;
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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    restaurant_list: Vec<String>
}

#[derive(Queryable)]
struct Restaurant {
    id: i32,
    name: String,
    last_visited_date: String,
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

fn main() {
    env_logger::init();

    let mut router = Router::new();
    router.get("/", index, "home");

    let mut chain = Chain::new(router);

    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("localhost:8080").unwrap();
}
