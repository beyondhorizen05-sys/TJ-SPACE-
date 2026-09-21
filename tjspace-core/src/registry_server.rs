use anyhow::{bail, Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path as FsPath, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tar::Archive;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    #[serde(default)] pub enabled: bool,
    #[serde(default = "default_bind")] pub bind: String,
    #[serde(default = "default_storage")] pub storage_dir: String,
    #[serde(default = "default_page_size")] pub default_page_size: usize,
    #[serde(default = "default_max_page_size")] pub max_page_size: usize,
    #[serde(default = "default_rate_capacity")] pub rate_capacity: u64,
    #[serde(default = "default_rate_refill")] pub rate_refill_per_second: f64,
    #[serde(default)] pub catalog_private_key_hex: Option<String>,
}
fn default_bind()->String{"127.0.0.1:8180".into()}
fn default_storage()->String{"tjspace-registry".into()}
fn default_page_one()->usize{1}
fn default_page_size()->usize{25}
fn default_max_page_size()->usize{100}
fn default_rate_capacity()->u64{120}
fn default_rate_refill()->f64{2.0}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogFilter {
    #[serde(default = "default_page_one")] pub page: usize,
    #[serde(default = "default_page_size")] pub page_size: usize,
    #[serde(default)] pub query: Option<String>,
    #[serde(default)] pub category: Option<String>,
    #[serde(default)] pub publisher: Option<String>,
    #[serde(default)] pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecord {
    pub package_id: String,
    pub version: String,
    pub title: String,
    pub license: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub publisher: String,
    pub publisher_key: String,
    pub artifact: String,
    pub signature: String,
    pub merkle_root: String,
    pub deprecated: bool,
    pub deprecation_reason: Option<String>,
    pub published_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogPage {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub packages: Vec<PackageRecord>,
    pub catalog_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCatalog {
    pub revision: u64,
    pub generated_at: u64,
    pub packages: Vec<PackageRecord>,
    pub merkle_root: String,
    pub publisher: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone)]
struct Bucket { tokens: f64, last: f64 }

#[derive(Clone)]
pub struct RegistryServer {
    pub config: RegistryConfig,
    records: Arc<Mutex<Vec<PackageRecord>>>,
    revision: Arc<Mutex<u64>>,
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    catalog_key: Arc<SigningKey>,
}

impl RegistryServer {
    pub fn new(config: RegistryConfig) -> Result<Self> {
        if config.max_page_size == 0 || config.default_page_size == 0 || config.default_page_size > config.max_page_size {
            bail!("invalid registry page-size limits");
        }
        if config.rate_capacity == 0 || config.rate_refill_per_second <= 0.0 { bail!("invalid rate-limit configuration"); }
        fs::create_dir_all(&config.storage_dir)?;
        let key = if let Some(hex_key) = &config.catalog_private_key_hex {
            let bytes = hex::decode(hex_key)?;
            ed25519_dalek::SigningKey::from_bytes(bytes.as_slice().try_into().map_err(|_| anyhow::anyhow!("catalog private key must be 32 bytes"))?)
        } else {
            let mut seed=[0u8;32]; let mut f=File::open("/dev/urandom").context("catalog key unavailable; configure catalog_private_key_hex")?; f.read_exact(&mut seed)?; SigningKey::from_bytes(&seed)
        };
        let catalog_path=PathBuf::from(&config.storage_dir).join("catalog.json");
        let persisted:Vec<PackageRecord>=if catalog_path.exists(){serde_json::from_slice(&fs::read(&catalog_path)?)?}else{Vec::new()};
        let revision=persisted.len() as u64;
        Ok(Self { config, records:Arc::new(Mutex::new(persisted)), revision:Arc::new(Mutex::new(revision)), buckets:Arc::new(Mutex::new(HashMap::new())), catalog_key:Arc::new(key) })
    }

    pub fn publish_package(&self, s9pk_path: &str, publisher_key: &[u8]) -> Result<PackageRecord> {
        crate::s9pk_toolchain::VerifyS9pk(s9pk_path, publisher_key)
            .context("publisher signature/integrity verification failed")?;
        let manifest = read_manifest_json(s9pk_path)?;
        let id = manifest["id"].as_str().context("manifest id required")?.to_owned();
        let title = manifest["title"].as_str().context("manifest title required")?.to_owned();
        let license = manifest["license"].as_str().context("manifest license required")?.to_owned();
        let version = manifest.get("version").and_then(Value::as_str).unwrap_or("0.0.0").to_owned();
        let category = manifest.get("category").and_then(Value::as_str).map(str::to_owned);
        let tags = manifest.get("tags").and_then(Value::as_array).map(|v| v.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
        let publisher = hex::encode(blake3::hash(publisher_key).as_bytes());
        let root = crate::s9pk_toolchain::ComputeMerkleRoot(s9pk_path)?;
        let signature = read_package_signature(s9pk_path)?;
        let artifact_name = format!("{}-{}.s9pk", safe(&id)?, safe(&version)?);
        let dest = PathBuf::from(&self.config.storage_dir).join(&artifact_name);
        fs::copy(s9pk_path, &dest)?;
        let record=PackageRecord{package_id:id.clone(),version:version.clone(),title,license,category,tags,publisher,publisher_key:hex::encode(publisher_key),artifact:artifact_name,signature,merkle_root:hex::encode(root),deprecated:false,deprecation_reason:None,published_at:now()};
        let mut records=self.records.lock().unwrap();
        if records.iter().any(|r|r.package_id==id && r.version==version) { bail!("package version already published"); }
        records.push(record.clone());
        *self.revision.lock().unwrap() += 1;
        let catalog_path=PathBuf::from(&self.config.storage_dir).join("catalog.json");
        fs::write(catalog_path,serde_json::to_vec_pretty(&*records)?)?;
        Ok(record)
    }

    pub fn get_index(&self, filter: CatalogFilter) -> Result<CatalogPage> {
        let page=filter.page.max(1);
        let page_size=filter.page_size.clamp(1,self.config.max_page_size);
        let records=self.records.lock().unwrap().clone();
        let filtered:Vec<_>=records.into_iter().filter(|r|matches_filter(r,&filter)).collect();
        let total=filtered.len();
        let start=(page-1).saturating_mul(page_size);
        let packages=filtered.into_iter().skip(start).take(page_size).collect();
        Ok(CatalogPage{page,page_size,total,packages,catalog_revision:*self.revision.lock().unwrap()})
    }

    pub fn get_package(&self, package_id: &str) -> Result<Vec<PackageRecord>> {
        let records=self.records.lock().unwrap();
        let out=records.iter().filter(|r|r.package_id==package_id).cloned().collect::<Vec<_>>();
        if out.is_empty(){bail!("package not found")} Ok(out)
    }
    pub fn get_package_version(&self, package_id:&str, version:&str)->Result<PackageRecord>{
        self.records.lock().unwrap().iter().find(|r|r.package_id==package_id&&r.version==version).cloned().ok_or_else(||anyhow::anyhow!("package version not found"))
    }
    pub fn get_signature(&self, package_id:&str, version:&str)->Result<String>{Ok(self.get_package_version(package_id,version)?.signature)}
    pub fn list_versions(&self, package_id:&str)->Result<Vec<String>>{let mut v=self.get_package(package_id)?.into_iter().map(|r|r.version).collect::<Vec<_>>();v.sort();Ok(v)}
    pub fn deprecate_version(&self, package_id:&str, version:&str, reason:&str)->Result<()>{
        if reason.trim().is_empty(){bail!("deprecation reason required")}
        let mut records=self.records.lock().unwrap();
        let r=records.iter_mut().find(|r|r.package_id==package_id&&r.version==version).ok_or_else(||anyhow::anyhow!("package version not found"))?;
        r.deprecated=true; r.deprecation_reason=Some(reason.to_owned()); *self.revision.lock().unwrap()+=1;
        let catalog_path=PathBuf::from(&self.config.storage_dir).join("catalog.json");
        fs::write(catalog_path,serde_json::to_vec_pretty(&*records)?)?; Ok(())
    }
    pub fn search_catalog(&self, query:&str)->Result<Vec<PackageRecord>>{
        let q=query.trim().to_lowercase();
        if q.len()>256 {bail!("query too long")}
        let records=self.records.lock().unwrap();
        Ok(records.iter().filter(|r|r.title.to_lowercase().contains(&q)||r.package_id.to_lowercase().contains(&q)||r.category.as_deref().unwrap_or("").to_lowercase().contains(&q)||r.publisher.to_lowercase().contains(&q)||r.tags.iter().any(|t|t.to_lowercase().contains(&q))).cloned().collect())
    }
    pub fn signed_catalog(&self)->Result<SignedCatalog>{
        let mut packages=self.records.lock().unwrap().clone();
        packages.sort_by(|a,b|a.package_id.cmp(&b.package_id).then(a.version.cmp(&b.version)));
        let revision=*self.revision.lock().unwrap();
        let canonical=serde_json::to_vec(&json!({"revision":revision,"packages":packages}))?;
        let root=*blake3::hash(&canonical).as_bytes();
        let sig=self.catalog_key.sign(&root);
        Ok(SignedCatalog{revision,generated_at:now(),packages,merkle_root:hex::encode(root),publisher:hex::encode(blake3::hash(self.catalog_key.verifying_key().as_bytes()).as_bytes()),public_key:hex::encode(self.catalog_key.verifying_key().as_bytes()),signature:base64::Engine::encode(&base64::engine::general_purpose::STANDARD,sig.to_bytes())})
    }

    pub async fn serve(self:Arc<Self>)->Result<()>{
        let app=Router::new()
            .route("/api/v1/registry/index",get(http_index))
            .route("/api/v1/registry/search",get(http_search))
            .route("/api/v1/registry/catalog.json",get(http_catalog))
            .route("/api/v1/registry/catalog-key",get(http_catalog_key))
            .route("/api/v1/registry/packages/:package_id",get(http_package))
            .route("/api/v1/registry/packages/:package_id/:version",get(http_version))
            .route("/api/v1/registry/packages/:package_id/:version/signature",get(http_signature))
            .route("/api/v1/registry/packages/:package_id/versions",get(http_versions))
            .route("/api/v1/registry/packages/:package_id/:version/deprecate",post(http_deprecate))
            .route("/api/v1/registry/publish",post(http_publish))
            .route("/api/v1/registry/artifacts/:artifact",get(http_artifact))
            .with_state(self.clone());
        let listener=TcpListener::bind(&self.config.bind).await?;
        axum::serve(listener,app).await?; Ok(())
    }
}

fn rate_limit(server:&RegistryServer, headers:&HeaderMap, route:&str)->Result<()>{
    let key=headers.get("x-api-key").and_then(|v|v.to_str().ok()).or_else(||headers.get("x-forwarded-for").and_then(|v|v.to_str().ok())).unwrap_or("anonymous");
    let key=format!("{}:{}",route,key);
    let now=now() as f64;
    let mut b=server.buckets.lock().unwrap();
    let e=b.entry(key).or_insert(Bucket{tokens:server.config.rate_capacity as f64,last:now});
    e.tokens=(e.tokens+(now-e.last)*server.config.rate_refill_per_second).min(server.config.rate_capacity as f64);
    e.last=now;
    if e.tokens<1.0 {b.shrink_to(server.config.rate_capacity as usize);bail!("rate limit exceeded")}
    e.tokens-=1.0; Ok(())
}
fn ok_or_429<T:Serialize>(r:Result<T>)->impl IntoResponse{match r{Ok(v)=>(StatusCode::OK,Json(v)).into_response(),Err(e) if e.to_string()=="rate limit exceeded"=>(StatusCode::TOO_MANY_REQUESTS,Json(json!({"error":"rate limit exceeded"}))).into_response(),Err(e)=>(StatusCode::BAD_REQUEST,Json(json!({"error":e.to_string()}))).into_response()}}
async fn http_index(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Query(q):Query<CatalogFilter>)->impl IntoResponse{let r=rate_limit(&s,&headers,"index").and_then(|_|s.get_index(q));ok_or_429(r)}
async fn http_search(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Query(q):Query<HashMap<String,String>>)->impl IntoResponse{let r=rate_limit(&s,&headers,"search").and_then(|_|s.search_catalog(q.get("q").map(String::as_str).unwrap_or("")));ok_or_429(r)}
async fn http_catalog(State(s):State<Arc<RegistryServer>>,headers:HeaderMap)->impl IntoResponse{let r=rate_limit(&s,&headers,"catalog").and_then(|_|s.signed_catalog());ok_or_429(r)}
async fn http_catalog_key(State(s):State<Arc<RegistryServer>>,headers:HeaderMap)->impl IntoResponse{let r=rate_limit(&s,&headers,"catalog-key").map(|_|json!({"publisher":hex::encode(blake3::hash(s.catalog_key.verifying_key().as_bytes()).as_bytes()),"public_key":hex::encode(s.catalog_key.verifying_key().as_bytes())}));ok_or_429(r)}
async fn http_package(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Path(id):Path<String>)->impl IntoResponse{let r=rate_limit(&s,&headers,"package").and_then(|_|s.get_package(&id));ok_or_429(r)}
async fn http_version(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Path((id,v)):Path<(String,String)>)->impl IntoResponse{let r=rate_limit(&s,&headers,"version").and_then(|_|s.get_package_version(&id,&v));ok_or_429(r)}
async fn http_signature(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Path((id,v)):Path<(String,String)>)->impl IntoResponse{let r=rate_limit(&s,&headers,"signature").and_then(|_|s.get_signature(&id,&v));ok_or_429(r)}
async fn http_versions(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Path(id):Path<String>)->impl IntoResponse{let r=rate_limit(&s,&headers,"versions").and_then(|_|s.list_versions(&id));ok_or_429(r)}
async fn http_deprecate(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Path((id,v)):Path<(String,String)>,Json(req):Json<DeprecationRequest>)->impl IntoResponse{let r=rate_limit(&s,&headers,"deprecate").and_then(|_|s.deprecate_version(&id,&v,&req.reason));ok_or_429(r)}
async fn http_publish(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Json(req):Json<PublishRequest>)->impl IntoResponse{
    let r=rate_limit(&s,&headers,"publish").and_then(|_|{if req.s9pk_base64.len()>128*1024*1024{bail!("artifact too large")}let bytes=base64::Engine::decode(&base64::engine::general_purpose::STANDARD,&req.s9pk_base64)?;let tmp=PathBuf::from(&s.config.storage_dir).join(format!(".upload-{}.s9pk",Uuid::new_v4()));fs::write(&tmp,&bytes)?;let out=s.publish_package(tmp.to_str().unwrap(),&hex::decode(req.publisher_key_hex)?);let _=fs::remove_file(&tmp);out});
    ok_or_429(r)
}
async fn http_artifact(State(s):State<Arc<RegistryServer>>,headers:HeaderMap,Path(artifact):Path<String>)->impl IntoResponse{
    let r=rate_limit(&s,&headers,"artifact").and_then(|_|{if artifact.contains('/')||artifact.contains('\\')||!artifact.ends_with(".s9pk"){bail!("invalid artifact name")}let p=PathBuf::from(&s.config.storage_dir).join(&artifact);if !p.exists(){bail!("artifact not found")}Ok(fs::read(p)?)});
    match r{Ok(bytes)=>(StatusCode::OK,bytes).into_response(),Err(e) if e.to_string()=="rate limit exceeded"=>(StatusCode::TOO_MANY_REQUESTS,Json(json!({"error":"rate limit exceeded"}))).into_response(),Err(_)=>(StatusCode::NOT_FOUND,Json(json!({"error":"artifact not found"}))).into_response()}
}
#[derive(Debug,Deserialize)] struct PublishRequest{s9pk_base64:String,publisher_key_hex:String}
#[derive(Debug,Deserialize)] struct DeprecationRequest{reason:String}

fn read_manifest_json(path:&str)->Result<Value>{let all=fs::read(path)?;if all.len()<15||all[..3]!=crate::s9pk_toolchain::S9PK_HEADER{bail!("invalid S9PK")}let mut p=3;if &all[p..p+4]!=b"S9IX"{bail!("invalid S9PK index")}p+=4;let mut n=[0u8;8];n.copy_from_slice(&all[p..p+8]);p+=8;let ilen=u64::from_le_bytes(n) as usize;if p+ilen>all.len(){bail!("truncated S9PK")}p+=ilen;let footer=all.windows(4).rposition(|w|w==b"S9SG").context("footer missing")?;if footer<p{bail!("invalid S9PK sections")}let body=&all[p..footer];let mut ar=Archive::new(body);for e in ar.entries()?{let mut e=e?;if e.path()?.to_string_lossy()=="startos/manifest/manifest.json"{let mut b=Vec::new();e.read_to_end(&mut b)?;return Ok(serde_json::from_slice(&b)?);}}bail!("manifest not found")}
fn read_package_signature(path:&str)->Result<String>{let bytes=fs::read(path)?;let pos=bytes.windows(4).rposition(|w|w==b"S9SG").context("signature footer missing")?;if pos+12>bytes.len(){bail!("truncated signature footer")}let mut n=[0u8;8];n.copy_from_slice(&bytes[pos+4..pos+12]);let len=u64::from_le_bytes(n) as usize;if pos+12+len>bytes.len(){bail!("invalid signature footer length")}let v:Value=serde_json::from_slice(&bytes[pos+12..pos+12+len])?;Ok(v["signature"].as_str().context("signature missing")?.to_owned())}
fn matches_filter(r:&PackageRecord,f:&CatalogFilter)->bool{
    let q=f.query.as_deref().unwrap_or("").to_lowercase();
    (q.is_empty()||r.title.to_lowercase().contains(&q)||r.package_id.to_lowercase().contains(&q)||r.publisher.to_lowercase().contains(&q)||r.tags.iter().any(|t|t.to_lowercase().contains(&q))) &&
    f.category.as_deref().map(|c|r.category.as_deref()==Some(c)).unwrap_or(true) &&
    f.publisher.as_deref().map(|p|r.publisher==p).unwrap_or(true) &&
    f.tag.as_deref().map(|t|r.tags.iter().any(|x|x==t)).unwrap_or(true)
}
fn safe(s:&str)->Result<String>{if s.is_empty()||s.len()>128||!s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_'||b==b'.'){bail!("unsafe identifier")}Ok(s.to_owned())}
fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs()}

#[cfg(test)]
mod tests{
use super::*; use tempfile::tempdir;
#[test]fn rate_limiter(){let d=tempdir().unwrap();let mut c=RegistryConfig{url:"http://x".into(),enabled:true,bind:"127.0.0.1:0".into(),storage_dir:d.path().display().to_string(),default_page_size:1,max_page_size:2,rate_capacity:2,rate_refill_per_second:1.0,catalog_private_key_hex:None};let s=RegistryServer::new(c.clone()).unwrap();let h=HeaderMap::new();assert!(rate_limit(&s,&h,"x").is_ok());assert!(rate_limit(&s,&h,"x").is_ok());assert!(rate_limit(&s,&h,"x").is_err());}
#[test]fn filter_search(){let d=tempdir().unwrap();let c=RegistryConfig{url:"http://x".into(),enabled:true,bind:"127.0.0.1:0".into(),storage_dir:d.path().display().to_string(),default_page_size:10,max_page_size:10,rate_capacity:10,rate_refill_per_second:1.0,catalog_private_key_hex:None};let s=RegistryServer::new(c).unwrap();s.records.lock().unwrap().push(PackageRecord{package_id:"demo".into(),version:"1.0.0".into(),title:"Demo App".into(),license:"MIT".into(),category:Some("tools".into()),tags:vec!["test".into()],publisher:"pub".into(),publisher_key:"00".into(),artifact:"demo-1.0.0.s9pk".into(),signature:"sig".into(),merkle_root:"root".into(),deprecated:false,deprecation_reason:None,published_at:now()});assert_eq!(s.search_catalog("demo").unwrap().len(),1);assert_eq!(s.get_index(CatalogFilter{page:1,page_size:1,query:None,category:Some("tools".into()),publisher:None,tag:None}).unwrap().total,1);}
}