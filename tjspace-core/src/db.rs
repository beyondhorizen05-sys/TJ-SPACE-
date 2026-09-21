use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect(url:&str)->Result<PgPool>{
    Ok(PgPoolOptions::new().max_connections(10).connect(url).await?)
}
pub async fn migrate(pool:&PgPool)->Result<()>{
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
