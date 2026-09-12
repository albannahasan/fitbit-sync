use sqlx::postgres::Postgres;
use sqlx::{PgPool, Pool};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = Pool::<Postgres>::connect(database_url).await?;
    return Ok(pool);
}
