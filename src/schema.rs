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
    charts (ticker, datetime) {
        #[max_length = 10]
        ticker -> Varchar,
        open -> Nullable<Float8>,
        high -> Nullable<Float8>,
        low -> Nullable<Float8>,
        close -> Nullable<Float8>,
        volume -> Nullable<Int4>,
        datetime -> Timestamp,
    }
}

diesel::table! {
    daily_cap (ticker) {
        #[max_length = 10]
        ticker -> Varchar,
        market_cap -> Nullable<Int8>,
        trading_volume -> Nullable<Int8>,
        change -> Nullable<Numeric>,
        datetime -> Timestamp,
    }
}

diesel::table! {
    interest (id) {
        id -> Uuid,
        sector_id -> Nullable<Uuid>,
        #[max_length = 200]
        symbol -> Nullable<Varchar>,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        price -> Nullable<Float8>,
        quantity -> Nullable<Float8>,
        order_type -> Nullable<Int4>,
        created_at -> Timestamp,
        #[max_length = 10]
        ticker -> Nullable<Varchar>,
    }
}

diesel::table! {
    positions (id) {
        id -> Uuid,
        #[max_length = 10]
        ticker -> Varchar,
        price -> Float8,
        quantity -> Float8,
        #[max_length = 10]
        strategy_id -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sector (id) {
        id -> Uuid,
        #[max_length = 200]
        name -> Nullable<Varchar>,
    }
}

diesel::table! {
    trading_volume (ticker, datetime) {
        #[max_length = 10]
        ticker -> Varchar,
        inst_sum -> Nullable<Numeric>,
        etc_corp -> Nullable<Int4>,
        individual -> Nullable<Int4>,
        foreigner -> Nullable<Int4>,
        total -> Nullable<Int4>,
        datetime -> Timestamp,
    }
}

diesel::joinable!(interest -> sector (sector_id));

diesel::allow_tables_to_appear_in_same_query!(
    articles,
    charts,
    daily_cap,
    interest,
    orders,
    positions,
    sector,
    trading_volume,
);
