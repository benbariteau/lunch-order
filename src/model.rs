use super::schema::{
    restaurant,
    user,
    user_private,
};

#[derive(Queryable)]
pub(crate) struct Restaurant {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) last_visit_time: String,
}

#[derive(Insertable)]
#[table_name = "restaurant"]
pub(crate) struct NewRestaurant {
    pub(crate) name: String,
    pub(crate) last_visit_time: String,
}

#[derive(Insertable)]
#[table_name = "user"]
pub(crate) struct NewUser {
    pub(crate) username: String,
}

#[derive(Insertable)]
#[table_name = "user_private"]
pub(crate) struct NewUserPrivate {
    pub(crate) user_id: i32,
    pub(crate) password_hash: String,
}
