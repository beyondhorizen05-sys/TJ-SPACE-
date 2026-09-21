pub mod backup;
pub mod bins;
pub mod db;
pub mod install;
pub mod lxc;
pub mod net;
pub mod os_install;
pub mod patch_db;
pub mod rpc_transport;
pub mod registry;
pub mod s9pk;
pub mod service;
pub mod sign;
pub mod tunnel;
pub mod update;
pub mod version;

use anyhow::Result;
use axum::{extract::State, http::{HeaderMap, StatusCode}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{collections::HashMap, future::Future, sync::Arc};
use tokio::sync::RwLock;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_bind")] pub bind: String,
    #[serde(default)] pub auth_token: String,
    #[serde(default)] pub runtime: RuntimeConfig,
    #[serde(default)] pub node_id: String,
    #[serde(default = "default_patch_db")] pub patch_db_path: String,
}
fn default_bind()->String{"127.0.0.1:8090".into()}
fn default_patch_db()->String{"tjspace-state.db".into()}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default)] pub dry_run: bool,
    #[serde(default = "default_timeout")] pub command_timeout_seconds: u64,
}
fn default_timeout()->u64{30}

#[derive(Debug, Clone, Serialize)]
pub struct SystemState {
    pub node_id:String,
    pub version:String,
    pub initialized:bool,
    pub services:Vec<service::ServiceRecord>,pub state_revision:u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcRequest {
    pub method:String,
    #[serde(default)] pub params:Value,
    #[serde(default)] pub trace_id:Option<String>,
    #[serde(skip)] pub auth_token:Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct RpcResponse {
    pub ok:bool,
    pub trace_id:String,
    pub result:Option<Value>,
    pub error:Option<RpcError>,
}
#[derive(Debug, Clone, Serialize)]
pub struct RpcError {pub code:String,pub message:String}

type BoxFuture<T> = std::pin::Pin<Box<dyn Future<Output=T> + Send + 'static>>;
type RpcHandler = Arc<dyn Fn(RpcRequest)->BoxFuture<Result<Value>> + Send + Sync>;

#[derive(Clone)]
pub struct Core {
    pub config:Config,
    pub db:PgPool,
    pub patch_db:patch_db::PatchDb,
    handlers:Arc<RwLock<HashMap<String,RpcHandler>>>,
}

impl Core {
    pub async fn init_core(config_path:&str)->Result<Arc<Self>>{
        let raw=tokio::fs::read_to_string(config_path).await?;
        let mut config:Config=serde_yaml::from_str(&raw)?;
        if config.node_id.is_empty(){config.node_id=Uuid::new_v4().to_string();}
        tracing_subscriber::fmt().json().with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_|"info".into())
        ).try_init().ok();
        let db=db::connect(&config.database_url).await?;
        db::migrate(&db).await?;let patch_db=patch_db::PatchDb::open_database(&config.patch_db_path)?;
        let core=Arc::new(Self{config,db,handlers:Arc::new(RwLock::new(HashMap::new())),patch_db});
        core.register_builtin_methods().await;
        tracing::info!(trace_id=%Uuid::new_v4(),service_id="tjsd","core_initialized");
        Ok(core)
    }

    pub async fn register_rpc_method<F,Fut>(&self,method:&str,handler:F)
    where F:Fn(RpcRequest)->Fut+Send+Sync+'static,Fut:Future<Output=Result<Value>>+Send+'static {
        let h:RpcHandler=Arc::new(move|req|Box::pin(handler(req)));
        self.handlers.write().await.insert(method.to_owned(),h);
    }

    pub async fn handle_rpc_request(&self,mut req:RpcRequest)->RpcResponse{
        let trace_id=req.trace_id.clone().unwrap_or_else(||Uuid::new_v4().to_string());
        let span=tracing::info_span!("rpc",trace_id=%trace_id,service_id="tjsd",method=%req.method);
        async{
            if !self.authorized(req.auth_token.as_deref()){
                return RpcResponse{ok:false,trace_id,result:None,error:Some(RpcError{code:"UNAUTHORIZED".into(),message:"invalid RPC credentials".into()})};
            }
            req.trace_id=Some(trace_id.clone());
            let handler=self.handlers.read().await.get(&req.method).cloned();
            match handler {
                Some(h)=>match h(req).await{
                    Ok(v)=>RpcResponse{ok:true,trace_id,result:Some(v),error:None},
                    Err(e)=>RpcResponse{ok:false,trace_id,result:None,error:Some(RpcError{code:"INTERNAL".into(),message:e.to_string()})},
                },
                None=>RpcResponse{ok:false,trace_id,result:None,error:Some(RpcError{code:"METHOD_NOT_FOUND".into(),message:"unknown RPC method".into()})},
            }
        }.instrument(span).await
    }

    fn authorized(&self,token:Option<&str>)->bool{
        self.config.auth_token.is_empty() || token==Some(self.config.auth_token.as_str())
    }
    pub async fn get_system_state(&self)->Result<SystemState>{
        Ok(SystemState{node_id:self.config.node_id.clone(),version:version::VERSION.into(),initialized:true,services:service::list(&self.db).await?,state_revision:self.patch_db.get_revision()?})
    }
    pub async fn start_service(&self,p:&str)->Result<()>{service::start(&self.db,&self.config.runtime,p).await}
    pub async fn stop_service(&self,p:&str,g:bool)->Result<()>{service::stop(&self.db,&self.config.runtime,p,g).await}
    pub async fn restart_service(&self,p:&str)->Result<()>{self.stop_service(p,true).await?;self.start_service(p).await}
    pub async fn install_package(&self,p:&str)->Result<String>{install::install(&self.db,&self.config.runtime,p).await}
    pub async fn uninstall_package(&self,p:&str,purge:bool)->Result<()>{install::uninstall(&self.db,&self.config.runtime,p,purge).await}
    pub async fn orchestrate_backup(&self,p:&str,t:&str)->Result<String>{backup::create(&self.db,p,t).await}
    pub async fn orchestrate_restore(&self,m:&str)->Result<()>{backup::restore(&self.db,m).await}
    pub async fn sign_artifact(&self,b:&[u8],k:&[u8])->Result<Vec<u8>>{sign::sign(b,k)}
    pub async fn verify_artifact(&self,b:&[u8],s:&[u8])->Result<()> {sign::verify(b,s)}

    async fn register_builtin_methods(self:&Arc<Self>){let c=self.clone();self.register_rpc_method("ApplyPatch",move|r|{let c=c.clone();async move{let p:patch_db::Patch=serde_json::from_value(r.params)?;Ok(serde_json::to_value(c.patch_db.apply_patch(p)?)?)}}).await;
        let c=self.clone();self.register_rpc_method("GetSystemState",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.get_system_state().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("StartService",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow::anyhow!("package_id required"))?;c.start_service(p).await?;Ok(json!({"started":p}))}}).await;
        let c=self.clone();self.register_rpc_method("StopService",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow::anyhow!("package_id required"))?;let g=r.params["graceful"].as_bool().unwrap_or(true);c.stop_service(p,g).await?;Ok(json!({"stopped":p,"graceful":g}))}}).await;
        let c=self.clone();self.register_rpc_method("RestartService",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow::anyhow!("package_id required"))?;c.restart_service(p).await?;Ok(json!({"restarted":p}))}}).await;
        let c=self.clone();self.register_rpc_method("InstallPackage",move|r|{let c=c.clone();async move{let p=r.params["s9pk_path"].as_str().ok_or_else(||anyhow::anyhow!("s9pk_path required"))?;Ok(json!({"package_id":c.install_package(p).await?}))}}).await;
        let c=self.clone();self.register_rpc_method("UninstallPackage",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow::anyhow!("package_id required"))?;let purge=r.params["purge_data"].as_bool().unwrap_or(false);c.uninstall_package(p,purge).await?;Ok(json!({"uninstalled":p}))}}).await;
        let c=self.clone();self.register_rpc_method("OrchestrateBackup",move|r|{let c=c.clone();async move{let p=r.params["package_id"].as_str().ok_or_else(||anyhow::anyhow!("package_id required"))?;let t=r.params["target_id"].as_str().ok_or_else(||anyhow::anyhow!("target_id required"))?;Ok(json!({"monolith_id":c.orchestrate_backup(p,t).await?}))}}).await;
        let c=self.clone();self.register_rpc_method("OrchestrateRestore",move|r|{let c=c.clone();async move{let m=r.params["monolith_id"].as_str().ok_or_else(||anyhow::anyhow!("monolith_id required"))?;c.orchestrate_restore(m).await?;Ok(json!({"restored":m}))}}).await;
    }

    pub async fn serve(self:Arc<Self>)->Result<()>{
        let app=Router::new().route("/api/v1/health",get(health)).route("/api/v1/rpc",post(rpc_http))
            .with_state(self.clone()).layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<axum::body::Body>| { let trace_id=request.headers().get("x-trace-id").and_then(|v|v.to_str().ok()).unwrap_or("generated"); tracing::info_span!("http_request",trace_id=%trace_id,service_id="tjsd",method=%request.method(),uri=%request.uri()) }));
        let listener=tokio::net::TcpListener::bind(&self.config.bind).await?;
        tracing::info!(trace_id=%Uuid::new_v4(),service_id="tjsd",bind=%self.config.bind,"rpc_server_started");
        axum::serve(listener,app).await?;
        Ok(())
    }
}
async fn health()->Json<Value>{Json(json!({"ok":true,"service_id":"tjsd"}))}
async fn rpc_http(State(core):State<Arc<Core>>,headers:HeaderMap,Json(mut req):Json<RpcRequest>)->(StatusCode,Json<RpcResponse>){
    req.auth_token=headers.get("authorization").and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer ")).map(str::to_owned);
    let out=core.handle_rpc_request(req).await;
    let code=if out.ok{StatusCode::OK}else if out.error.as_ref().map(|e|e.code.as_str())==Some("UNAUTHORIZED"){StatusCode::UNAUTHORIZED}else{StatusCode::BAD_REQUEST};
    (code,Json(out))
}
