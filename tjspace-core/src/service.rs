use anyhow::{anyhow,Result};
use serde::Serialize;
use sqlx::PgPool;
use crate::{lxc,RuntimeConfig};

#[derive(Debug,Clone,Serialize,sqlx::FromRow)]
pub struct ServiceRecord{
    pub package_id:String,
    pub state:String,
    pub updated_at:chrono::DateTime<chrono::Utc>
}
pub async fn list(db:&PgPool)->Result<Vec<ServiceRecord>>{
    Ok(sqlx::query_as::<_,ServiceRecord>("SELECT package_id,state,updated_at FROM services ORDER BY package_id").fetch_all(db).await?)
}
async fn ensure_exists(db:&PgPool,pkg:&str)->Result<()>{
    let ok=sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM services WHERE package_id=$1)")
        .bind(pkg).fetch_one(db).await?;
    if !ok { return Err(anyhow!("service not installed: {pkg}")) }
    Ok(())
}
pub async fn start(db:&PgPool,cfg:&RuntimeConfig,pkg:&str)->Result<()>{
    ensure_exists(db,pkg).await?;
    lxc::start(cfg,pkg).await?;
    sqlx::query("UPDATE services SET state='running',updated_at=now() WHERE package_id=$1").bind(pkg).execute(db).await?;
    tracing::info!(trace_id=%uuid::Uuid::new_v4(),service_id=%pkg,"service_started");
    Ok(())
}
pub async fn stop(db:&PgPool,cfg:&RuntimeConfig,pkg:&str,graceful:bool)->Result<()>{
    ensure_exists(db,pkg).await?;
    lxc::stop(cfg,pkg,graceful).await?;
    sqlx::query("UPDATE services SET state='stopped',updated_at=now() WHERE package_id=$1").bind(pkg).execute(db).await?;
    tracing::info!(trace_id=%uuid::Uuid::new_v4(),service_id=%pkg,graceful,"service_stopped");
    Ok(())
}
