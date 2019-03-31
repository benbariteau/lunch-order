table! {
    restaurant {
        id -> Integer,
        name -> VarChar,
        last_visit_time -> VarChar,
    }
}

table! {
    user {
        id -> Integer,
        name -> VarChar,
    }
}

table! {
    user_private {
        id -> Integer,
        user_id -> Integer,
        password_hash -> VarChar,
    }
}
