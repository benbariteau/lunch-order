use super::schema::restaurant;

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
