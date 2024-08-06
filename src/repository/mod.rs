use sea_orm::{entity::*, query::*};
use anyhow::Result;
use sea_orm::{Database, DatabaseConnection, QueryFilter};
use crate::models::positions::Model;
use crate::models::prelude::*;
pub struct Repository {
    pool: DatabaseConnection,
}

impl Repository {
    pub async fn new(database_url: String) -> Result<Self, sea_orm::DbErr> {
        let pool = Database::connect(&database_url).await?;
        Ok(Self { pool })
    }

    pub async fn get_positions(&self) -> Result<Vec<Model>> {
        let result = Positions::find().all(&self.pool).await?;
        Ok(result)
    }

    pub async fn create_order(&self,order: Orders) -> Result<()> {
        Orders::insert_many()

    }

}

#[cfg(test)]
mod test {
    use diesel_async::RunQueryDsl;
    use super::*;
    #[tokio::test]
    async fn test_crud_order() {
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string()).await.unwrap();
        let result = repo.get_positions().await.unwrap();



    }


}