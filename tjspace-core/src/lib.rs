pub mod backup;
pub mod bins;
pub mod db;
pub mod install;
pub mod hardware;
pub mod lxc;
pub mod net;
pub mod os_install;
pub mod os_installer;
pub mod os_updates;
pub mod patch_db;
pub mod rpc_transport;
pub mod container_runtime;
pub mod registry;
pub mod registry_server;
pub mod s9pk;
pub mod s9pk_toolchain;
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
    #[serde(default)] pub hardware: hardware::HardwareConfig,
    #[serde(default)] pub node_id: String,
    #[serde(default = "default_patch_db")] pub patch_db_path: String,
    #[serde(default)] pub os_updates: os_updates::OsUpdateConfig,
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
    pub hardware: hardware::HardwareManager,
    pub installer: os_installer::OsInstaller,
    pub os_updates: os_updates::OsUpdateManager,
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
        let hardware_config=config.hardware.clone();
        let installer=os_installer::OsInstaller::new(patch_db.clone(),os_installer::InstallerConfig{dry_run:config.runtime.dry_run,command_timeout_seconds:config.runtime.command_timeout_seconds,live_mode_required:true});
        let os_updates=os_updates::OsUpdateManager::new(patch_db.clone(), config.os_updates.clone());
        let _=os_updates.RecoverFailedBoot();
        let core=Arc::new(Self{config,db,patch_db:patch_db.clone(),hardware:hardware::HardwareManager::new(patch_db.clone(),hardware_config),installer,os_updates,handlers:Arc::new(RwLock::new(HashMap::new()))});
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
    pub async fn start_service(&self,p:&str)->Result<()>{if self.os_updates.IsSafeMode()?{return Err(anyhow::anyhow!("safe mode: service start denied"))}service::start(&self.db,&self.config.runtime,p).await}
    pub async fn stop_service(&self,p:&str,g:bool)->Result<()>{service::stop(&self.db,&self.config.runtime,p,g).await}
    pub async fn restart_service(&self,p:&str)->Result<()>{self.stop_service(p,true).await?;self.start_service(p).await}
    pub async fn install_package(&self,p:&str)->Result<String>{install::install(&self.db,&self.config.runtime,p).await}
    pub async fn uninstall_package(&self,p:&str,purge:bool)->Result<()>{install::uninstall(&self.db,&self.config.runtime,p,purge).await}
    pub async fn orchestrate_backup(&self,p:&str,t:&str)->Result<String>{backup::create(&self.db,p,t).await}
    pub async fn orchestrate_restore(&self,m:&str)->Result<()>{backup::restore(&self.db,m).await}
    pub async fn sign_artifact(&self,b:&[u8],k:&[u8])->Result<Vec<u8>>{sign::sign(b,k)}
    pub async fn verify_artifact(&self,b:&[u8],s:&[u8])->Result<()> {sign::verify(b,s)}

    async fn register_builtin_methods(self:&Arc<Self>){
        let c=self.clone();self.register_rpc_method("RunInstaller",move|r|{let c=c.clone();async move{let disk=r.params["target_disk"].as_str().ok_or_else(||anyhow::anyhow!("target_disk required"))?;let opts:os_installer::InstallOptions=serde_json::from_value(r.params["options"].clone())?;Ok(serde_json::to_value(c.installer.run_installer(disk,&opts).await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("PartitionDisk",move|r|{let c=c.clone();async move{let disk=r.params["disk_id"].as_str().ok_or_else(||anyhow::anyhow!("disk_id required"))?;let layout:os_installer::PartitionLayout=serde_json::from_value(r.params["layout"].clone())?;Ok(serde_json::to_value(c.installer.partition_disk(disk,&layout)?)?)}}).await;
        let c=self.clone();self.register_rpc_method("WriteBaseSystem",move|r|{let c=c.clone();async move{let p=r.params["partition"].as_str().ok_or_else(||anyhow::anyhow!("partition required"))?;let s=r.params["source"].as_str().ok_or_else(||anyhow::anyhow!("source required"))?;c.installer.write_base_system(p,s).await?;Ok(json!({"written":true}))}}).await;
        let c=self.clone();self.register_rpc_method("ConfigureBootloader",move|r|{let c=c.clone();async move{let d=r.params["disk_id"].as_str().ok_or_else(||anyhow::anyhow!("disk_id required"))?;c.installer.configure_bootloader(d)?;Ok(json!({"configured":true}))}}).await;
        let c=self.clone();self.register_rpc_method("FirstBootWizard",move|_|{let c=c.clone();async move{c.installer.first_boot_wizard().await?;Ok(json!({"configured":true}))}}).await;
        let c=self.clone();self.register_rpc_method("RestoreFromBackup",move|r|{let c=c.clone();async move{let s=r.params["backup_source"].as_str().ok_or_else(||anyhow::anyhow!("backup_source required"))?;c.installer.restore_from_backup(s).await?;Ok(json!({"restored":true}))}}).await;
        let c=self.clone();self.register_rpc_method("VerifyInstallation",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.installer.verify_installation().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("RequestInstallerConfirmation",move|r|{let c=c.clone();async move{let op=r.params["operation"].as_str().ok_or_else(||anyhow::anyhow!("operation required"))?;let res=r.params["resource"].as_str().ok_or_else(||anyhow::anyhow!("resource required"))?;Ok(json!({"operation_id":c.installer.request_confirmation(op,res)?}))}}).await;
        let c=self.clone();self.register_rpc_method("ConfirmInstallerOperation",move|r|{let c=c.clone();async move{let id=r.params["operation_id"].as_str().ok_or_else(||anyhow::anyhow!("operation_id required"))?;c.installer.confirm(id)?;Ok(json!({"confirmed":true}))}}).await;
        let c=self.clone();self.register_rpc_method("CheckForOsUpdate",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.os_updates.CheckForOsUpdate().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("DownloadOsUpdate",move|r|{let c=c.clone();async move{let id=r.params["release_id"].as_str().ok_or_else(||anyhow::anyhow!("release_id required"))?;Ok(json!({"path":c.os_updates.DownloadOsUpdate(id).await?.display().to_string()}))}}).await;
        let c=self.clone();self.register_rpc_method("VerifyOsUpdate",move|r|{let c=c.clone();async move{let id=r.params["release_id"].as_str().ok_or_else(||anyhow::anyhow!("release_id required"))?;Ok(json!({"verified":c.os_updates.VerifyOsUpdate(id).await?}))}}).await;
        let c=self.clone();self.register_rpc_method("ApplyOsUpdate",move|r|{let c=c.clone();async move{let id=r.params["release_id"].as_str().ok_or_else(||anyhow::anyhow!("release_id required"))?;Ok(serde_json::to_value(c.os_updates.ApplyOsUpdate(id).await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("EnterSafeMode",move|_|{let c=c.clone();async move{c.os_updates.EnterSafeMode()?;Ok(json!({"safe_mode":true}))}}).await;
        let c=self.clone();self.register_rpc_method("ExitSafeMode",move|_|{let c=c.clone();async move{c.os_updates.ExitSafeMode()?;Ok(json!({"safe_mode":false}))}}).await;
        let c=self.clone();self.register_rpc_method("FactoryReset",move|r|{let c=c.clone();async move{let p=r.params["preserve_data"].as_bool().unwrap_or(false);Ok(serde_json::to_value(c.os_updates.FactoryReset(p)?)?)}}).await;
        let c=self.clone();self.register_rpc_method("RollbackOs",move|r|{let c=c.clone();async move{let v=r.params["target_version"].as_str().ok_or_else(||anyhow::anyhow!("target_version required"))?;c.os_updates.RollbackOs(v)?;Ok(json!({"rolled_back_to":v}))}}).await;
        let c=self.clone();self.register_rpc_method("GetUpdateHistory",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.os_updates.GetUpdateHistory()?)?)}}).await;
        let c=self.clone();self.register_rpc_method("BootHealthAck",move|r|{let c=c.clone();async move{let v=r.params["version"].as_str().ok_or_else(||anyhow::anyhow!("version required"))?;c.os_updates.BootHealthAck(v)?;Ok(json!({"acknowledged":true}))}}).await;
        let c=self.clone();self.register_rpc_method("EnumerateDisks",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.hardware.EnumerateDisks().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("GetDiskHealth",move|r|{let c=c.clone();async move{let id=r.params["disk_id"].as_str().ok_or_else(||anyhow::anyhow!("disk_id required"))?;Ok(serde_json::to_value(c.hardware.GetDiskHealth(id).await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("GetSystemHardware",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.hardware.GetSystemHardware().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("ScanWifiNetworks",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.hardware.ScanWifiNetworks().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("GetDrivers",move|_|{let c=c.clone();async move{Ok(serde_json::to_value(c.hardware.GetDrivers().await?)?)}}).await;
        let c=self.clone();self.register_rpc_method("CreateVolume",move|r|{let c=c.clone();async move{let name=r.params["name"].as_str().ok_or_else(||anyhow::anyhow!("name required"))?;let disks:Vec<String>=serde_json::from_value(r.params["disks"].clone())?;let raid=r.params["raid_level"].as_str().ok_or_else(||anyhow::anyhow!("raid_level required"))?;let fs=r.params["fs_type"].as_str().ok_or_else(||anyhow::anyhow!("fs_type required"))?;let v=if let Some(id)=r.params["confirmation_id"].as_str(){c.hardware.CreateVolumeConfirmed(name,&disks,raid,fs,id).await?}else{c.hardware.CreateVolume(name,&disks,raid,fs).await?};Ok(serde_json::to_value(v)?)}}).await;
        let c=self.clone();self.register_rpc_method("ResizeVolume",move|r|{let c=c.clone();async move{let id=r.params["volume_id"].as_str().ok_or_else(||anyhow::anyhow!("volume_id required"))?;let size=r.params["new_size"].as_u64().ok_or_else(||anyhow::anyhow!("new_size required"))?;let v=if let Some(cid)=r.params["confirmation_id"].as_str(){c.hardware.ResizeVolumeConfirmed(id,size,cid).await?}else{c.hardware.ResizeVolume(id,size).await?};Ok(serde_json::to_value(v)?)}}).await;
        let c=self.clone();self.register_rpc_method("DeleteVolume",move|r|{let c=c.clone();async move{let id=r.params["volume_id"].as_str().ok_or_else(||anyhow::anyhow!("volume_id required"))?;if let Some(cid)=r.params["confirmation_id"].as_str(){c.hardware.DeleteVolumeConfirmed(id,cid).await?}else{c.hardware.DeleteVolume(id).await?};Ok(json!({"deleted":id}))}}).await;
        let c=self.clone();self.register_rpc_method("MountVolume",move|r|{let c=c.clone();async move{let id=r.params["volume_id"].as_str().ok_or_else(||anyhow::anyhow!("volume_id required"))?;let mp=r.params["mount_point"].as_str().ok_or_else(||anyhow::anyhow!("mount_point required"))?;c.hardware.MountVolume(id,mp).await?;Ok(json!({"mounted":id,"mount_point":mp}))}}).await;
        let c=self.clone();self.register_rpc_method("UnmountVolume",move|r|{let c=c.clone();async move{let id=r.params["volume_id"].as_str().ok_or_else(||anyhow::anyhow!("volume_id required"))?;c.hardware.UnmountVolume(id).await?;Ok(json!({"unmounted":id}))}}).await;
        let c=self.clone();self.register_rpc_method("ConfigureWifi",move|r|{let c=c.clone();async move{let ssid=r.params["ssid"].as_str().ok_or_else(||anyhow::anyhow!("ssid required"))?;let credentials:hardware::WifiCredentials=serde_json::from_value(r.params["credentials"].clone())?;c.hardware.ConfigureWifi(ssid,&credentials).await?;Ok(json!({"configured":ssid}))}}).await;
        let c=self.clone();self.register_rpc_method("LoadDriver",move|r|{let c=c.clone();async move{let module=r.params["module"].as_str().ok_or_else(||anyhow::anyhow!("module required"))?;c.hardware.LoadDriver(module).await?;Ok(json!({"loaded":module}))}}).await;
        let c=self.clone();self.register_rpc_method("UnloadDriver",move|r|{let c=c.clone();async move{let module=r.params["module"].as_str().ok_or_else(||anyhow::anyhow!("module required"))?;if let Some(cid)=r.params["confirmation_id"].as_str(){c.hardware.UnloadDriverConfirmed(module,cid).await?}else{c.hardware.UnloadDriver(module).await?};Ok(json!({"unloaded":module}))}}).await;
        let c=self.clone();self.register_rpc_method("RequestHardwareConfirmation",move|r|{let c=c.clone();async move{let op=r.params["operation"].as_str().ok_or_else(||anyhow::anyhow!("operation required"))?;let resource=r.params["resource"].as_str().ok_or_else(||anyhow::anyhow!("resource required"))?;Ok(serde_json::to_value(c.hardware.request_confirmation(op,resource)?)?)}}).await;
        let c=self.clone();self.register_rpc_method("ConfirmHardwareOperation",move|r|{let c=c.clone();async move{let id=r.params["operation_id"].as_str().ok_or_else(||anyhow::anyhow!("operation_id required"))?;c.hardware.confirm_destructive_operation(id)?;Ok(json!({"confirmed":id}))}}).await;
let c=self.clone();self.register_rpc_method("ApplyPatch",move|r|{let c=c.clone();async move{let p:patch_db::Patch=serde_json::from_value(r.params)?;Ok(serde_json::to_value(c.patch_db.apply_patch(p)?)?)}}).await;
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
