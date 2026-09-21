use anyhow::{anyhow, Result};
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, middleware::{self, Next}, response::Response, routing::{get, post}, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use clap::{Args, Parser, Subcommand};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::{collections::BTreeMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use crate::{Core, RpcRequest, RpcResponse};

type HmacSha256 = Hmac<Sha256>;

pub const EXIT_OK:i32=0;
pub const EXIT_USAGE:i32=2;
pub const EXIT_AUTH:i32=3;
pub const EXIT_NOT_FOUND:i32=4;
pub const EXIT_CONFLICT:i32=5;
pub const EXIT_PERMISSION:i32=6;
pub const EXIT_IO:i32=7;
pub const EXIT_REMOTE:i32=8;
pub const EXIT_INTERNAL:i32=70;

#[derive(Debug,Clone,Serialize,Deserialize)]
struct JwtClaims{sub:String,exp:u64,iat:u64,scope:String}
fn b64(v:&[u8])->String{URL_SAFE_NO_PAD.encode(v)}
pub fn issue_jwt(secret:&str,subject:&str,scope:&str,ttl:u64)->Result<String>{
    if secret.is_empty(){return Err(anyhow!("JWT secret is not configured"))}
    let now=SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let head=b64(br#"{"alg":"HS256","typ":"JWT"}"#);
    let body=b64(&serde_json::to_vec(&JwtClaims{sub:subject.into(),exp:now+ttl,iat:now,scope:scope.into()})?);
    let input=format!("{head}.{body}");
    let mut mac=HmacSha256::new_from_slice(secret.as_bytes())?;mac.update(input.as_bytes());
    Ok(format!("{input}.{}",b64(&mac.finalize().into_bytes())))
}
macro_rules! bail_auth {()=>{return Err(anyhow!("invalid JWT"))};}\nfn verify_jwt(secret:&str,token:&str)->Result<JwtClaims>{
    let p:Vec<_>=token.split('.').collect();if p.len()!=3{bail_auth!();}
    let input=format!("{}.{}",p[0],p[1]);let mut mac=HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_|anyhow!("JWT secret invalid"))?;
    mac.update(input.as_bytes());let sig=URL_SAFE_NO_PAD.decode(p[2])?;mac.verify_slice(&sig).map_err(|_|anyhow!("invalid JWT"))?;
    let claims:JwtClaims=serde_json::from_slice(&URL_SAFE_NO_PAD.decode(p[1])?)?;
    let now=SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();if claims.exp<=now{bail_auth!();}Ok(claims)
}
macro_rules! bail_auth {()=>{return Err(anyhow!("invalid JWT"))};}

#[derive(Debug,Serialize,Deserialize)]
pub struct ApiEnvelope{pub method:String,#[serde(default)]pub params:Value,#[serde(default)]pub trace_id:Option<String>}
#[derive(Debug,Serialize,Deserialize)]
pub struct ApiError{pub code:String,pub message:String}

pub async fn register_rpc(core:&Arc<Core>){
    let c=core.clone();core.register_rpc_method("ListServices",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(crate::service::list(&c.db).await?)?)}}).await;
    let c=core.clone();core.register_rpc_method("ServiceLogs",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow!("package_id required"))?;let n=r.params["lines"].as_u64().unwrap_or(100).min(5000);let out=tokio::process::Command::new("journalctl").args(["-u",&format!("tjspace-{p}"),"-n",&n.to_string(),"--no-pager","-o","cat"]).output().await?;Ok(json!({"package_id":p,"lines":String::from_utf8_lossy(&out.stdout)}))}}).await;
    let c=core.clone();core.register_rpc_method("ServiceConfig",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow!("package_id required"))?;let row=sqlx::query_as::<_,(String,String)>("SELECT package_id,source_path FROM packages WHERE package_id=$1").bind(p).fetch_optional(&c.db).await?.ok_or_else(||anyhow!("service not found"))?;Ok(json!({"package_id":row.0,"source_path":row.1}))}}).await;
    let c=core.clone();core.register_rpc_method("ListPackages",move|_|{let c=c.clone();async move{let rows=sqlx::query_as::<_,(String,String,String,bool)>("SELECT package_id,version,source_path,data_purged FROM packages ORDER BY package_id").fetch_all(&c.db).await?;Ok(serde_json::to_value(rows)?)}}).await;
    let c=core.clone();core.register_rpc_method("InspectPackage",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow!("package_id required"))?;let path=sqlx::query_scalar::<_,String>("SELECT source_path FROM packages WHERE package_id=$1").bind(p).fetch_optional(&c.db).await?.ok_or_else(||anyhow!("package not found"))?;Ok(serde_json::to_value(crate::s9pk_toolchain::ParseManifest(&path)?)?)}}).await;
    let c=core.clone();core.register_rpc_method("VerifyPackage",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow!("package_id required"))?;let path=sqlx::query_scalar::<_,String>("SELECT source_path FROM packages WHERE package_id=$1").bind(p).fetch_optional(&c.db).await?.ok_or_else(||anyhow!("package not found"))?;let key=hex::decode(r.params["public_key_hex"].as_str().ok_or_else(||anyhow!("public_key_hex required"))?)?;crate::s9pk_toolchain::VerifyS9pk(&path,&key)?;Ok(json!({"verified":true,"package_id":p}))}}).await;
    let c=core.clone();core.register_rpc_method("ListBackups",move|_|{let c=c.clone();async move{let rows=sqlx::query("SELECT monolith_id,package_id,target_id,state FROM backups ORDER BY monolith_id").fetch_all(&c.db).await?;let out=rows.iter().map(|r|json!({"monolith_id":r.get::<String,_>("monolith_id"),"package_id":r.get::<String,_>("package_id"),"target_id":r.get::<String,_>("target_id"),"state":r.get::<String,_>("state")})).collect::<Vec<_>>();Ok(json!(out))}}).await;
    let c=core.clone();core.register_rpc_method("VerifyBackup",move|r|{let c=c.clone();async move{let id=r.params["monolith_id"].as_str().ok_or_else(||anyhow!("monolith_id required"))?;let ok=sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM backups WHERE monolith_id=$1 AND state='current')").bind(id).fetch_one(&c.db).await?;Ok(json!({"monolith_id":id,"verified":ok}))}}).await;
    let c=core.clone();core.register_rpc_method("ListVolumes",move|_|{let c=c.clone();async move{Ok(c.hardware.get_state_snapshot()?.state.get("hardware").and_then(|v|v.get("volumes")).cloned().unwrap_or(json!({})) )}}).await;
    let c=core.clone();core.register_rpc_method("RegistryAdd",move|r|{let c=c.clone();async move{let url=r.params["url"].as_str().ok_or_else(||anyhow!("url required"))?;let key=format!("registry.endpoints.{}",blake3::hash(url.as_bytes()).to_hex());c.patch_db.apply_patch(crate::patch_db::Patch{version:1,path:key,op:crate::patch_db::PatchOp::Set,value:Some(json!({"url":url,"added_at":chrono::Utc::now()})),actor:"tjsd".into(),authorization:"allow".into()})?;Ok(json!({"url":url,"added":true}))}}).await;
    let c=core.clone();core.register_rpc_method("RegistryRemove",move|r|{let c=c.clone();async move{let url=r.params["url"].as_str().ok_or_else(||anyhow!("url required"))?;let key=format!("registry.endpoints.{}",blake3::hash(url.as_bytes()).to_hex());c.patch_db.apply_patch(crate::patch_db::Patch{version:1,path:key,op:crate::patch_db::PatchOp::Delete,value:None,actor:"tjsd".into(),authorization:"allow".into()})?;Ok(json!({"url":url,"removed":true}))}}).await;
    let c=core.clone();core.register_rpc_method("RegistryList",move|_|{let c=c.clone();async move{let s=c.patch_db.get_snapshot()?;Ok(s.state.get("registry").and_then(|v|v.get("endpoints")).cloned().unwrap_or(json!({})))}}).await;
    let c=core.clone();core.register_rpc_method("RegistrySearch",move|r|{let c=c.clone();async move{let q=r.params["query"].as_str().ok_or_else(||anyhow!("query required"))?;let s=c.patch_db.get_snapshot()?;let eps=s.state.get("registry").and_then(|v|v.get("endpoints")).cloned().unwrap_or(json!({}));Ok(json!({"query":q,"registries":eps}))}}).await;
    let c=core.clone();core.register_rpc_method("DeviceAdd",move|r|{let c=c.clone();async move{let id=r.params["device_id"].as_str().ok_or_else(||anyhow!("device_id required"))?;let key=format!("devices.{}",id);c.patch_db.apply_patch(crate::patch_db::Patch{version:1,path:key,op:crate::patch_db::PatchOp::Set,value:Some(json!({"device_id":id,"status":"active","added_at":chrono::Utc::now()})),actor:"tjsd".into(),authorization:"allow".into()})?;Ok(json!({"device_id":id,"added":true}))}}).await;
    let c=core.clone();core.register_rpc_method("DeviceRevoke",move|r|{let c=c.clone();async move{let id=r.params["device_id"].as_str().ok_or_else(||anyhow!("device_id required"))?;let key=format!("devices.{}",id);c.patch_db.apply_patch(crate::patch_db::Patch{version:1,path:key,op:crate::patch_db::PatchOp::Set,value:Some(json!({"device_id":id,"status":"revoked","revoked_at":chrono::Utc::now()})),actor:"tjsd".into(),authorization:"allow".into()})?;Ok(json!({"device_id":id,"revoked":true}))}}).await;
    let c=core.clone();core.register_rpc_method("DeviceList",move|_|{let c=c.clone();async move{Ok(c.patch_db.get_snapshot()?.state.get("devices").cloned().unwrap_or(json!({})))}}).await;
    let c=core.clone();core.register_rpc_method("DiagDump",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.patch_db.get_snapshot()?)?)}}).await;
    let c=core.clone();core.register_rpc_method("DiagRestore",move|r|{let c=c.clone();async move{let snap:serde_json::Value=r.params["snapshot"].clone();let obj=snap.get("state").cloned().unwrap_or(snap);for (k,v) in flatten_json("",&obj){if !k.is_empty(){c.patch_db.apply_patch(crate::patch_db::Patch{version:1,path:k,op:crate::patch_db::PatchOp::Set,value:Some(v),actor:"tjsd".into(),authorization:"allow".into()})?;}}Ok(json!({"restored":true}))}}).await;
}

fn flatten_json(prefix:&str,v:&Value)->Vec<(String,Value)>{let mut out=Vec::new();match v{Value::Object(m)=>for(k,x)in m{let p=if prefix.is_empty(){k.clone()}else{format!("{prefix}.{k}")};out.extend(flatten_json(&p,x));},_=>out.push((prefix.into(),v.clone()))}out}

pub async fn serve_api(core:Arc<Core>)->Result<()>{
    let app=Router::new()
      .route("/api/v1/status",get(api_get_status))
      .route("/api/v1/service/:action",post(api_dispatch))
      .route("/api/v1/package/:action",post(api_dispatch))
      .route("/api/v1/registry/:action",post(api_dispatch))
      .route("/api/v1/volume/:action",post(api_dispatch))
      .route("/api/v1/backup/:action",post(api_dispatch))
      .route("/api/v1/device/:action",post(api_dispatch))
      .route("/api/v1/update/:action",post(api_dispatch))
      .route("/api/v1/diag/:action",post(api_dispatch))
      .with_state(core.clone())
      .layer(middleware::from_fn_with_state(core.clone(),jwt_middleware));
    let listener=tokio::net::TcpListener::bind(&core.config.bind).await?;axum::serve(listener,app).await?;Ok(())
}
async fn jwt_middleware(State(core):State<Arc<Core>>,headers:HeaderMap,req:axum::extract::Request,next:Next)->Result<Response,StatusCode>{
    let token=headers.get("authorization").and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer ")).ok_or(StatusCode::UNAUTHORIZED)?;
    verify_jwt(&core.config.jwt_secret,token).map_err(|_|StatusCode::UNAUTHORIZED)?;Ok(next.run(req).await)
}
async fn api_get_status(State(core):State<Arc<Core>>)->Result<Json<Value>,StatusCode>{core.get_system_state().await.map(|v|Json(serde_json::to_value(v).unwrap())).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)}
async fn api_dispatch(State(core):State<Arc<Core>>,Path(action):Path<String>,Json(mut env):Json<ApiEnvelope>)->Result<Json<Value>,StatusCode>{
    env.method=method_for_path(&env.method,&action);let out=core.handle_rpc_request(RpcRequest{method:env.method,params:env.params,trace_id:env.trace_id,auth_token:Some(core.config.auth_token.clone())}).await;
    if out.ok{Ok(Json(out.result.unwrap_or(Value::Null)))}else{Err(if out.error.as_ref().map(|e|e.code.as_str())==Some("METHOD_NOT_FOUND"){StatusCode::NOT_FOUND}else{StatusCode::BAD_REQUEST})}
}
fn method_for_path(method:&str,action:&str)->String{if method.is_empty(){match action{_=>String::new()}}else{method.into()}}

#[derive(Parser,Debug)]
#[command(name="tjspace-cli",version,about="TJ SPACE headless control CLI")]
pub struct Cli{#[arg(long,default_value="http://127.0.0.1:8090")]pub endpoint:String,#[arg(long)]pub token:Option<String>,#[arg(long)]pub json:bool,#[arg(long)]pub quiet:bool,#[command(subcommand)]pub command:CliCommand}
#[derive(Subcommand,Debug)]
pub enum CliCommand{
 Status,
 Service(ServiceCommand),Package(PackageCommand),Registry(RegistryCommand),Volume(VolumeCommand),Backup(BackupCommand),Device(DeviceCommand),Update(UpdateCommand),Diag(DiagCommand),
}
#[derive(Subcommand,Debug)]pub enum ServiceCommand{List,Start{package_id:String},Stop{package_id:String,#[arg(long)]force:bool},Restart{package_id:String},Logs{package_id:String,#[arg(long,default_value_t=100)]lines:u64},Config{package_id:String}}
#[derive(Subcommand,Debug)]pub enum PackageCommand{Install{path:String},Uninstall{package_id:String,#[arg(long)]purge_data:bool},List,Inspect{package_id:String},Verify{package_id:String,#[arg(long)]public_key_hex:String}}
#[derive(Subcommand,Debug)]pub enum RegistryCommand{Add{url:String},Remove{url:String},List,Search{query:String}}
#[derive(Subcommand,Debug)]pub enum VolumeCommand{Create{request:std::path::PathBuf,#[arg(long)]confirmation_id:Option<String>},Resize{volume_id:String,new_size:u64,#[arg(long)]confirmation_id:Option<String>},Delete{volume_id:String,#[arg(long)]confirmation_id:Option<String>},List}
#[derive(Subcommand,Debug)]pub enum BackupCommand{Create{package_id:String,target_id:String},Restore{monolith_id:String},List,Verify{monolith_id:String}}
#[derive(Subcommand,Debug)]pub enum DeviceCommand{Add{device_id:String},Revoke{device_id:String},List}
#[derive(Subcommand,Debug)]pub enum UpdateCommand{Check,Apply{release_id:String},Rollback{target_version:String}}
#[derive(Subcommand,Debug)]pub enum DiagCommand{Dump,Restore{snapshot:std::path::PathBuf}}

pub async fn run_cli(cli:Cli)->i32{
    let token=match cli.token.or_else(||std::env::var("TJS_JWT_TOKEN").ok()){Some(t)=>t,None=>{eprintln!("authentication token required");return EXIT_AUTH}};
    let client=Client::new();
    let call=|method:String,params:Value|{let client=client.clone();let endpoint=cli.endpoint.clone();let token=token.clone();async move{
        let req=ApiEnvelope{method,params,trace_id:None};let r=client.post(format!("{endpoint}/api/v1/rpc")).bearer_auth(token).json(&req).send().await?;let status=r.status();let v:RpcResponse=r.json().await?;if !status.is_success()||!v.ok{return Err(anyhow!(v.error.map(|e|e.message).unwrap_or_else(||format!("HTTP {status}"))))}Ok(v.result.unwrap_or(Value::Null))}}};
    let result=match cli.command{
        CliCommand::Status=>call("GetSystemState".into(),json!({})).await,
        CliCommand::Service(c)=>service_call(&call,c).await,
        CliCommand::Package(c)=>package_call(&call,c).await,
        CliCommand::Registry(c)=>registry_call(&call,c).await,
        CliCommand::Volume(c)=>volume_call(&call,c).await,
        CliCommand::Backup(c)=>backup_call(&call,c).await,
        CliCommand::Device(c)=>device_call(&call,c).await,
        CliCommand::Update(c)=>update_call(&call,c).await,
        CliCommand::Diag(c)=>diag_call(&call,c).await,
    };
    match result{Ok(v)=>{if !cli.quiet{if cli.json{println!("{}",serde_json::to_string(&v).unwrap())}else{println!("{}",serde_json::to_string_pretty(&v).unwrap())}}EXIT_OK},Err(e)=>{if !cli.quiet{eprintln!("{e}")}EXIT_REMOTE}}
}
async fn service_call<F,Fut>(call:&F,c:ServiceCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{ServiceCommand::List=>call("ListServices".into(),json!({})).await,ServiceCommand::Start{package_id}=>call("StartService".into(),json!({"package_id":package_id})).await,ServiceCommand::Stop{package_id,force}=>call("StopService".into(),json!({"package_id":package_id,"graceful":!force})).await,ServiceCommand::Restart{package_id}=>call("RestartService".into(),json!({"package_id":package_id})).await,ServiceCommand::Logs{package_id,lines}=>call("ServiceLogs".into(),json!({"package_id":package_id,"lines":lines})).await,ServiceCommand::Config{package_id}=>call("ServiceConfig".into(),json!({"package_id":package_id})).await}}
async fn package_call<F,Fut>(call:&F,c:PackageCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{PackageCommand::Install{path}=>call("InstallPackage".into(),json!({"s9pk_path":path})).await,PackageCommand::Uninstall{package_id,purge_data}=>call("UninstallPackage".into(),json!({"package_id":package_id,"purge_data":purge_data})).await,PackageCommand::List=>call("ListPackages".into(),json!({})).await,PackageCommand::Inspect{package_id}=>call("InspectPackage".into(),json!({"package_id":package_id})).await,PackageCommand::Verify{package_id,public_key_hex}=>call("VerifyPackage".into(),json!({"package_id":package_id,"public_key_hex":public_key_hex})).await}}
async fn registry_call<F,Fut>(call:&F,c:RegistryCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{RegistryCommand::Add{url}=>call("RegistryAdd".into(),json!({"url":url})).await,RegistryCommand::Remove{url}=>call("RegistryRemove".into(),json!({"url":url})).await,RegistryCommand::List=>call("RegistryList".into(),json!({})).await,RegistryCommand::Search{query}=>call("RegistrySearch".into(),json!({"query":query})).await}}
async fn volume_call<F,Fut>(call:&F,c:VolumeCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{VolumeCommand::Create{request,confirmation_id}=>{let v:Value=serde_json::from_slice(&tokio::fs::read(request).await?)?;let mut p=v.as_object().cloned().unwrap_or_default();if let Some(i)=confirmation_id{p.insert("confirmation_id".into(),Value::String(i));}call("CreateVolume".into(),Value::Object(p)).await},VolumeCommand::Resize{volume_id,new_size,confirmation_id}=>call("ResizeVolume".into(),json!({"volume_id":volume_id,"new_size":new_size,"confirmation_id":confirmation_id})).await,VolumeCommand::Delete{volume_id,confirmation_id}=>call("DeleteVolume".into(),json!({"volume_id":volume_id,"confirmation_id":confirmation_id})).await,VolumeCommand::List=>call("ListVolumes".into(),json!({})).await}}
async fn backup_call<F,Fut>(call:&F,c:BackupCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{BackupCommand::Create{package_id,target_id}=>call("OrchestrateBackup".into(),json!({"package_id":package_id,"target_id":target_id})).await,BackupCommand::Restore{monolith_id}=>call("OrchestrateRestore".into(),json!({"monolith_id":monolith_id})).await,BackupCommand::List=>call("ListBackups".into(),json!({})).await,BackupCommand::Verify{monolith_id}=>call("VerifyBackup".into(),json!({"monolith_id":monolith_id})).await}}
async fn device_call<F,Fut>(call:&F,c:DeviceCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{DeviceCommand::Add{device_id}=>call("DeviceAdd".into(),json!({"device_id":device_id})).await,DeviceCommand::Revoke{device_id}=>call("DeviceRevoke".into(),json!({"device_id":device_id})).await,DeviceCommand::List=>call("DeviceList".into(),json!({})).await}}
async fn update_call<F,Fut>(call:&F,c:UpdateCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{UpdateCommand::Check=>call("CheckForOsUpdate".into(),json!({})).await,UpdateCommand::Apply{release_id}=>call("ApplyOsUpdate".into(),json!({"release_id":release_id})).await,UpdateCommand::Rollback{target_version}=>call("RollbackOs".into(),json!({"target_version":target_version})).await}}
async fn diag_call<F,Fut>(call:&F,c:DiagCommand)->Result<Value>where F:Fn(String,Value)->Fut,Fut:std::future::Future<Output=Result<Value>>{match c{DiagCommand::Dump=>call("DiagDump".into(),json!({})).await,DiagCommand::Restore{snapshot}=>{let v:Value=serde_json::from_slice(&tokio::fs::read(snapshot).await?)?;call("DiagRestore".into(),json!({"snapshot":v})).await}}}
