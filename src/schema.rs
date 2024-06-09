// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vector"))]
    pub struct Vector;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Vector;

    article (id) {
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
    orders (id) {
        #[max_length = 200]
        id -> Varchar,
        price -> Nullable<Float8>,
        amount -> Nullable<Float8>,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    article,
    orders,
);
