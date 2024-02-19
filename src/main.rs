#![allow(dead_code)]
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[derive(Debug)]
struct Batch {
    id: Option<i32>,
    sku: Option<String>,
}

impl Batch {
    fn new(sku: String) -> Batch {
        Batch {
            id: None,
            sku: Some(sku),
        }
    }
}

#[async_trait]
trait BatchRepository {
    async fn create_batch(&self, sku: String) -> anyhow::Result<i32>;
    async fn read_batch(&self, id: i32) -> anyhow::Result<Batch>;
    async fn update_batch(&self, id: i32, sku: String) -> anyhow::Result<bool>;
    async fn delete_batch(&self, id: i32) -> anyhow::Result<bool>;
}

struct PostgresBatchRepository {
    pg_pool: Arc<PgPool>,
}

impl PostgresBatchRepository {
    fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }
}

#[async_trait]
impl BatchRepository for PostgresBatchRepository {
    async fn create_batch(&self, sku: String) -> anyhow::Result<i32> {
        let record = sqlx::query!(
            r#"
                INSERT INTO batches (sku)
                VALUES ( $1 )
                RETURNING id
            "#,
            sku
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(record.id)
    }

    async fn read_batch(&self, id: i32) -> anyhow::Result<Batch> {
        let result = sqlx::query_as!(
            Batch,
            r#"SELECT id, sku FROM batches WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await;

        match result {
            Ok(batch) => Ok(batch),
            Err(err) => Err(anyhow::anyhow!("batch id: {} msg: {}", id, err)),
        }
    }

    async fn update_batch(&self, id: i32, sku: String) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query!(
            r#"UPDATE batches SET sku = $1 WHERE id = $2"#,
            sku,
            id
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn delete_batch(&self, id: i32) -> anyhow::Result<bool> {
        let rows_affected =
            sqlx::query!(r#"DELETE FROM batches WHERE id = $1"#, id)
                .execute(&*self.pg_pool)
                .await?
                .rows_affected();

        Ok(rows_affected > 0)
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // load variables from .env
    dotenvy::dotenv().expect("Failed to load .env file");

    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL not defined");

    let pool = PgPool::connect(&db_url).await?;
    let repo = PostgresBatchRepository::new(pool);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_create_and_read() {
        let pg_pool =
            PgPool::connect("postgresql://postgres:postgres@localhost:5432")
                .await
                .expect("Unable to connect to DB");

        sqlx::query("DROP TABLE IF EXISTS batches")
            .execute(&pg_pool)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE batches (id SERIAL PRIMARY KEY, sku VARCHAR(255))",
        )
        .execute(&pg_pool)
        .await
        .unwrap();

        let repo = PostgresBatchRepository {
            pg_pool: Arc::new(pg_pool),
        };

        assert_eq!(1, repo.create_batch("TEST".to_string()).await.unwrap());
    }
}
