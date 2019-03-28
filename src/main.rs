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
use iron::{
    Chain,
    Iron,
    IronResult,
    Request,
    Response,
    status,
};
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
        post {
            id -> Integer,
            name -> VarChar,
            last_visited_date -> VarChar,
        }
    }
}

fn create_db_connection() -> ConnectionResult<SqliteConnection> {
    SqliteConnection::establish("lunch_order.db")
}

fn get_restaurants() -> errors::LunchOrderResult<Vec<Restaurant>> {
    let db_connection = create_db_connection()?;
    let restaurants: Vec<Restaurant> = schema::post::table
        .order(schema::post::last_visited_date.desc())
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
    let mut router = Router::new();
    router.get("/", index, "home");
    let mut chain = Chain::new(router);
    Iron::new(chain).http("localhost:8080").unwrap();
}
