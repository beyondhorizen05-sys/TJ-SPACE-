use anyhow::Result;
use sqlx::PgPool;
use crate::{lxc,RuntimeConfig,s9pk};

pub async fn install(db:&PgPool,_cfg:&RuntimeConfig,path:&str)->Result<String>{
    let m=s9pk::read_manifest(path)?;
    sqlx::query("INSERT INTO packages(package_id,version,source_path) VALUES($1,$2,$3) ON CONFLICT(package_id) DO UPDATE SET version=EXCLUDED.version,source_path=EXCLUDED.source_path,data_purged=FALSE")
        .bind(&m.package_id).bind(&m.version).bind(path).execute(db).await?;
    sqlx::query("INSERT INTO services(package_id,state) VALUES($1,'stopped') ON CONFLICT(package_id) DO NOTHING")
        .bind(&m.package_id).execute(db).await?;
    tracing::info!(trace_id=%uuid::Uuid::new_v4(),service_id=%m.package_id,version=%m.version,"package_installed");
    Ok(m.package_id)
}
pub async fn uninstall(db:&PgPool,cfg:&RuntimeConfig,pkg:&str,purge:bool)->Result<()>{
    lxc::stop(cfg,pkg,true).await.ok();
    lxc::remove(cfg,pkg).await?;
    if purge {
        sqlx::query("DELETE FROM packages WHERE package_id=$1").bind(pkg).execute(db).await?;
    } else {
        sqlx::query("UPDATE packages SET data_purged=FALSE WHERE package_id=$1").bind(pkg).execute(db).await?;
        sqlx::query("DELETE FROM services WHERE package_id=$1").bind(pkg).execute(db).await?;
    }
    tracing::info!(trace_id=%uuid::Uuid::new_v4(),service_id=%pkg,purge,"package_uninstalled");
    Ok(())
}
