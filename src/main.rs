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
use std::path::Path;

mod model;
mod schema;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    logged_in: bool,
    restaurant_list: Vec<RestaurantPresenter>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {}

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

fn create_user(user: &model::NewUser) -> errors::LunchOrderResult<i32> {
    let db_connection = create_db_connection()?;
    let user_id = db_connection.transaction(|| -> errors::LunchOrderResult<i32> {
        diesel::insert_into(schema::user::table)
            .values(user)
            .execute(&db_connection)?;
        let user_id = schema::user::table
            .select(schema::user::id)
            .order(schema::user::id.desc())
            .first(&db_connection)?;
        Ok(user_id)
    })?;
    Ok(user_id)
}

fn create_user_private(user_private: &model::NewUserPrivate) -> errors::LunchOrderResult<()> {
    let db_connection = create_db_connection()?;
    diesel::insert_into(schema::user_private::table)
        .values(user_private)
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
            IndexTemplate{
                logged_in: false,
                restaurant_list: restaurant_presenters,
            },
        ))
    )
}

#[derive(Deserialize)]
struct NewRestaurantForm {
    name: String,
}

fn add_restaurant(request: &mut Request) -> IronResult<Response> {
    let restaurant_form: NewRestaurantForm = itry!(serde_urlencoded::from_reader(&mut request.body));
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

fn register_form(_request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((
        status::Ok,
        RegisterTemplate{},
    )))
}

#[derive(Deserialize)]
struct NewUserForm {
    username: String,
    password: String,
}

fn register(request: &mut Request) -> IronResult<Response> {
    let user_form: NewUserForm = itry!(serde_urlencoded::from_reader(&mut request.body));
    let password_hash = itry!(bcrypt::hash(user_form.password, bcrypt::DEFAULT_COST));
    let user_id = itry!(create_user(&model::NewUser{username: user_form.username}));
    itry!(create_user_private(&model::NewUserPrivate{user_id: user_id, password_hash: password_hash}));
    Ok(Response::with((
        status::SeeOther,
        RedirectRaw("/".to_owned()),
    )))
}


fn main() {
    env_logger::init();

    let mut router = Router::new();
    router.get("/", index, "home");
    router.post("/restaurant", add_restaurant, "add_restaurant");
    router.post("/visit/:id", visit, "visit");
    router.get("/register", register_form, "register_form");
    router.post("/register", register, "register");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static/", staticfile::Static::new(Path::new("static/")));

    let mut chain = Chain::new(mount);

    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("localhost:8080").unwrap();
}
