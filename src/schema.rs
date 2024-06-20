// @generated automatically by Diesel CLI.

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
