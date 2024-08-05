use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
#[derive(Serialize, Deserialize, Clone)]
#[derive(Insertable, Selectable, Queryable, Debug)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Position {
    pub id: uuid::Uuid,
    pub ticker: String,
    quantity: f64,
    price: f64,
    pub strategy_id: String,
    created_at: chrono::NaiveDateTime,
}

impl Position {
    pub fn new(id: uuid::Uuid, ticker: String, quantity: f64, price: f64, strategy_id: String) -> Self {
        Self {
            id,
            ticker,
            quantity,
            price,
            strategy_id,
            created_at: chrono::Local::now().naive_local(),
        }
    }

    pub fn quantity(&self) -> f64 {
        self.quantity
    }

    pub fn price(&self) -> f64 {
        self.price
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    pub fn update_quantity(&mut self, new_quantity: f64) {
        self.quantity = new_quantity;
    }

    pub fn update_price(&mut self, new_price: f64) {
        self.price = new_price;
    }

    pub fn value(&self) -> f64 {
        self.quantity * self.price
    }
}