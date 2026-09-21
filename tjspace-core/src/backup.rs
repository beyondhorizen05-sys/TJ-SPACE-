use anyhow::{anyhow,Result};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(db:&PgPool,pkg:&str,target:&str)->Result<String>{
    let exists=sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM packages WHERE package_id=$1)")
        .bind(pkg).fetch_one(db).await?;
    if !exists{return Err(anyhow!("package not installed"))}
    let id=Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO backups(monolith_id,package_id,target_id,state) VALUES($1,$2,$3,'current')")
        .bind(&id).bind(pkg).bind(target).execute(db).await?;
    tracing::info!(trace_id=%Uuid::new_v4(),service_id=%pkg,monolith_id=%id,target_id=%target,"backup_created");
    Ok(id)
}
pub async fn restore(db:&PgPool,id:&str)->Result<()>{
    let pkg=sqlx::query_scalar::<_,String>("SELECT package_id FROM backups WHERE monolith_id=$1 AND state='current'")
        .bind(id).fetch_optional(db).await?.ok_or_else(||anyhow!("restore point not found"))?;
    tracing::info!(trace_id=%Uuid::new_v4(),service_id=%pkg,monolith_id=%id,"restore_orchestrated");
    Ok(())
}
