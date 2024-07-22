// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vector"))]
    pub struct Vector;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Vector;

    articles (id) {
        #[max_length = 200]
        id -> Varchar,
        #[max_length = 200]
        title -> Nullable<Varchar>,
        contents -> Nullable<Text>,
        embedding -> Nullable<Vector>,
        #[max_length = 200]
        url -> Nullable<Varchar>,
        #[max_length = 200]
        site -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    interest (id) {
        #[max_length = 20]
        id -> Varchar,
        #[max_length = 20]
        sector_id -> Nullable<Varchar>,
        #[max_length = 200]
        symbol -> Nullable<Varchar>,
    }
}

diesel::table! {
    orders (id) {
        #[max_length = 200]
        id -> Varchar,
        price -> Nullable<Float8>,
        amount -> Nullable<Float8>,
        order_type -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sector (id) {
        #[max_length = 20]
        id -> Varchar,
        #[max_length = 200]
        name -> Nullable<Varchar>,
    }
}

diesel::joinable!(interest -> sector (sector_id));

diesel::allow_tables_to_appear_in_same_query!(articles, interest, orders, sector,);
