use crate::domain::model::Batch;

trait BatchRepo<'a>: Send + Sync {
    fn get_batch(&self, id: u32) -> Option<Batch>;
    fn save_batch(&self, batch: &Batch<'a>);
}

#[cfg(test)]
mod test {
    use sqlx::PgPool;

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
