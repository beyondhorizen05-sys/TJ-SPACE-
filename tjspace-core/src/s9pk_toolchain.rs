use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::{self, File}, io::{Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}};
use tar::{Archive, Builder, Header};
use uuid::Uuid;

pub const S9PK_HEADER: [u8; 3] = [0x3b, 0x3b, 0x02];
const INDEX_MAGIC: &[u8; 4] = b"S9IX";
const FOOTER_MAGIC: &[u8; 4] = b"S9SG";
const CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S9pkManifest {
    pub id: String,
    pub title: String,
    pub license: String,
    #[serde(default)] pub images: Vec<ImageSpec>,
    #[serde(default)] pub volumes: Vec<Value>,
    #[serde(default)] pub dependencies: Vec<Value>,
    #[serde(default)] pub architecture: Vec<String>,
    #[serde(default)] pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSpec {
    pub name: String,
    pub path: String,
    #[serde(default)] pub architectures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageIndex {
    files: Vec<IndexEntry>,
    merkle_root: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    path: String,
    offset: u64,
    length: u64,
    hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureFooter {
    merkle_root: String,
    signature: String,
}

pub fn build_s9pk(project_dir: impl AsRef<Path>, output_path: impl AsRef<Path>) -> Result<()> {
    let project_dir = project_dir.as_ref();
    let output_path = output_path.as_ref();
    let manifest_path = project_dir.join("startos/manifest/manifest.json");
    let manifest = if manifest_path.exists() {
        parse_manifest_file(&manifest_path)?
    } else {
        let y = project_dir.join("startos/manifest/manifest.yaml");
        if !y.exists() { bail!("missing startos/manifest/manifest.json or manifest.yaml"); }
        serde_yaml::from_str(&fs::read_to_string(y)?)?
    };
    validate_manifest(&manifest)?;
    let mut files = Vec::<(String, Vec<u8>)>::new();
    collect_files(project_dir, project_dir, &mut files)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    files.retain(|(p, _)| p != "startos/manifest/manifest.json");
    files.push(("startos/manifest/manifest.json".into(), manifest_bytes));
    files.sort_by(|a,b| a.0.cmp(&b.0));

    let mut body = Vec::new();
    {
        let mut builder = Builder::new(&mut body);
        for (path, data) in &files {
            let mut h = Header::new_gnu();
            h.set_path(path)?;
            h.set_size(data.len() as u64);
            h.set_mode(0o644); h.set_mtime(0); h.set_cksum();
            builder.append(&h, data.as_slice())?;
        }
        builder.finish()?;
    }
    let entries = tar_entries(&body)?;
    let root = merkle_root(&entries);
    let index = PackageIndex { files: entries.clone(), merkle_root: hex::encode(root) };
    let index_bytes = serde_json::to_vec(&index)?;
    let mut out = Vec::with_capacity(3 + 8 + index_bytes.len() + body.len() + 8);
    out.extend_from_slice(&S9PK_HEADER);
    out.extend_from_slice(INDEX_MAGIC);
    out.extend_from_slice(&(index_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&index_bytes);
    out.extend_from_slice(&body);
    out.extend_from_slice(FOOTER_MAGIC);
    out.extend_from_slice(&0u64.to_le_bytes());
    if let Some(parent) = output_path.parent() { fs::create_dir_all(parent)?; }
    fs::write(output_path, out)?;
    Ok(())
}

pub fn parse_manifest(s9pk_path: impl AsRef<Path>) -> Result<S9pkManifest> {
    let body = read_body(s9pk_path.as_ref())?;
    let mut ar = Archive::new(body.as_slice());
    for entry in ar.entries()? {
        let mut e = entry?;
        if e.path()?.to_string_lossy() == "startos/manifest/manifest.json" {
            let mut bytes = Vec::new(); e.read_to_end(&mut bytes)?;
            return Ok(serde_json::from_slice(&bytes)?);
        }
    }
    bail!("manifest not found")
}

pub fn compute_merkle_root(s9pk_path: impl AsRef<Path>) -> Result<[u8; 32]> {
    let (_, entries) = read_index(s9pk_path.as_ref())?;
    Ok(merkle_root(&entries))
}

pub fn sign_s9pk(s9pk_path: impl AsRef<Path>, private_key: &[u8]) -> Result<()> {
    let path=s9pk_path.as_ref();
    let (_, entries)=read_index(path)?;
    let root=merkle_root(&entries);
    let key=SigningKey::from_bytes(private_key.try_into().map_err(|_| anyhow::anyhow!("private key must be 32 bytes"))?);
    let sig=key.sign(&root);
    let footer=SignatureFooter{merkle_root:hex::encode(root),signature:base64::Engine::encode(&base64::engine::general_purpose::STANDARD,sig.to_bytes())};
    let bytes=serde_json::to_vec(&footer)?;
    let mut f=File::options().read(true).write(true).open(path)?;
    f.seek(SeekFrom::End(-8))?;
    f.write_all(&(bytes.len() as u64).to_le_bytes())?;
    f.seek(SeekFrom::End(0))?;
    f.write_all(&bytes)?;
    Ok(())
}

pub fn verify_s9pk(s9pk_path: impl AsRef<Path>, public_key: &[u8]) -> Result<()> {
    let path=s9pk_path.as_ref();
    let (index, entries)=read_index(path)?;
    let body=read_body(path)?;
    let actual=tar_entries(&body)?;
    if actual.len()!=entries.len() { bail!("package entry count mismatch"); }
    for (expected, found) in entries.iter().zip(actual.iter()) { if expected.path!=found.path || expected.hash!=found.hash || expected.length!=found.length { bail!("package content integrity mismatch: {}", expected.path); } }
    let computed=merkle_root(&actual);
    if hex::encode(computed) != index.merkle_root { bail!("Merkle root mismatch"); }
    let footer=read_footer(path)?;
    if footer.merkle_root != hex::encode(computed) { bail!("signature root mismatch"); }
    let key=VerifyingKey::from_bytes(public_key.try_into().map_err(|_| anyhow::anyhow!("public key must be 32 bytes"))?)?;
    let sig_bytes=base64::Engine::decode(&base64::engine::general_purpose::STANDARD,footer.signature)?;
    let sig=Signature::from_slice(&sig_bytes)?;
    key.verify(&computed,&sig).context("invalid S9PK signature")?;
    Ok(())
}

pub fn extract_partial(s9pk_path: impl AsRef<Path>, chunk_range: std::ops::Range<u64>) -> Result<Vec<u8>> {
    let mut f=File::open(s9pk_path)?;
    let len=f.metadata()?.len();
    let start=chunk_range.start.checked_mul(CHUNK_SIZE as u64).context("range overflow")?;
    let end=chunk_range.end.checked_mul(CHUNK_SIZE as u64).context("range overflow")?.min(len);
    if start>end || start>len { bail!("invalid chunk range"); }
    f.seek(SeekFrom::Start(start))?;
    let mut out=vec![0u8;(end-start) as usize];
    f.read_exact(&mut out)?;
    Ok(out)
}

pub fn BuildS9pk(project_dir: &str, output_path: &str) -> Result<()> { build_s9pk(project_dir, output_path) }
pub fn ParseManifest(s9pk_path: &str) -> Result<S9pkManifest> { parse_manifest(s9pk_path) }
pub fn ComputeMerkleRoot(s9pk_path: &str) -> Result<[u8;32]> { compute_merkle_root(s9pk_path) }
pub fn SignS9pk(s9pk_path: &str, private_key: &[u8]) -> Result<()> { sign_s9pk(s9pk_path, private_key) }
pub fn VerifyS9pk(s9pk_path: &str, public_key: &[u8]) -> Result<()> { verify_s9pk(s9pk_path, public_key) }
pub fn ExtractPartial(s9pk_path: &str, chunk_range: std::ops::Range<u64>) -> Result<Vec<u8>> { extract_partial(s9pk_path, chunk_range) }

fn parse_manifest_file(path:&Path)->Result<S9pkManifest>{Ok(serde_json::from_str(&fs::read_to_string(path)?)?)}
fn validate_manifest(m:&S9pkManifest)->Result<()>{
    if m.id.is_empty() || m.title.is_empty() || m.license.is_empty(){bail!("manifest id/title/license required")}
    if !m.id.bytes().all(|b| b.is_ascii_alphanumeric()||b==b'-'||b==b'_'||b==b'.'){bail!("invalid manifest id")}
    for img in &m.images { if img.name.is_empty() || img.path.is_empty(){bail!("image name/path required")} }
    Ok(())
}
fn collect_files(root:&Path, dir:&Path, out:&mut Vec<(String,Vec<u8>)>)->Result<()>{
    for e in fs::read_dir(dir)? { let e=e?; let p=e.path(); if p.file_name().map(|n|n==".git").unwrap_or(false){continue}
        if p.is_dir(){collect_files(root,&p,out)?} else if p.is_file(){let rel=p.strip_prefix(root)?.to_string_lossy().replace('\\',"/"); if !rel.ends_with(".s9pk"){out.push((rel,fs::read(p)?));}}
    } Ok(())
}
fn tar_entries(body:&[u8])->Result<Vec<IndexEntry>>{
    let mut ar=Archive::new(body); let mut entries=Vec::new();
    for e in ar.entries()? { let mut e=e?; let path=e.path()?.to_string_lossy().into_owned(); let offset=e.raw_file_position(); let mut data=Vec::new(); e.read_to_end(&mut data)?; entries.push(IndexEntry{path,offset,length:data.len() as u64,hash:hex::encode(blake3::hash(&data).as_bytes())}); }
    Ok(entries)
}
fn merkle_root(entries:&[IndexEntry])->[u8;32]{
    if entries.is_empty(){return *blake3::hash(b"").as_bytes();}
    let mut level:Vec<[u8;32]>=entries.iter().map(|e|{let mut h=blake3::Hasher::new();h.update(b"tjs-s9pk-leaf\0");h.update(e.path.as_bytes());h.update(&[0]);h.update(e.hash.as_bytes());*h.finalize().as_bytes()}).collect();
    while level.len()>1 { let mut next=Vec::with_capacity((level.len()+1)/2); for pair in level.chunks(2){let mut h=blake3::Hasher::new();h.update(b"tjs-s9pk-node\0");h.update(&pair[0]);if pair.len()==2{h.update(&pair[1])}else{h.update(&pair[0])};next.push(*h.finalize().as_bytes())} level=next; }
    level[0]
}
fn read_index(path:&Path)->Result<(PackageIndex,Vec<IndexEntry>)>{
    let mut f=File::open(path)?; let mut head=[0u8;3]; f.read_exact(&mut head)?; if head!=S9PK_HEADER{bail!("invalid S9PK header")}
    let mut magic=[0u8;4]; f.read_exact(&mut magic)?; if magic!=*INDEX_MAGIC{bail!("missing S9PK index")}
    let mut n=[0u8;8]; f.read_exact(&mut n)?; let len=u64::from_le_bytes(n) as usize; if len>16*1024*1024{bail!("index too large")}
    let mut bytes=vec![0;len]; f.read_exact(&mut bytes)?; let idx:PackageIndex=serde_json::from_slice(&bytes)?; Ok((idx.clone(),idx.files))
}
fn read_body(path:&Path)->Result<Vec<u8>>{let mut f=File::open(path)?;let mut all=Vec::new();f.read_to_end(&mut all)?;let (_,_,body)=split_sections(&all)?;Ok(body)}
fn read_footer(path:&Path)->Result<SignatureFooter>{let mut f=File::open(path)?;let len=f.metadata()?.len();if len<12{bail!("truncated package")}f.seek(SeekFrom::End(-8))?;let mut n=[0;8];f.read_exact(&mut n)?;let flen=u64::from_le_bytes(n);if flen==0||flen>len-12{bail!("invalid footer length")}let start=len-8-flen;f.seek(SeekFrom::Start(start-4))?;let mut magic=[0;4];f.read_exact(&mut magic)?;if magic!=*FOOTER_MAGIC{bail!("invalid footer magic")}let mut b=vec![0;flen as usize];f.read_exact(&mut b)?;Ok(serde_json::from_slice(&b)?)} 
fn split_sections(all:&[u8])->Result<(PackageIndex,usize,Vec<u8>)>{if all.len()<15||all[..3]!=S9PK_HEADER{bail!("invalid S9PK header")}let mut p=3;if &all[p..p+4]!=INDEX_MAGIC{bail!("invalid S9PK index")};p+=4;let mut n=[0;8];n.copy_from_slice(&all[p..p+8]);p+=8;let ilen=u64::from_le_bytes(n) as usize;if p+ilen>all.len(){bail!("truncated index")}let idx:PackageIndex=serde_json::from_slice(&all[p..p+ilen])?;p+=ilen;let body_end=all.windows(4).rposition(|w|w==FOOTER_MAGIC).context("footer missing")?;Ok((idx,p,all[p..body_end].to_vec()))}

#[cfg(test)]
mod tests{
 use super::*; use tempfile::tempdir; use rand_core::OsRng;
 #[test] fn build_parse_merkle(){let d=tempdir().unwrap();fs::create_dir_all(d.path().join("startos/manifest")).unwrap();fs::write(d.path().join("startos/manifest/manifest.json"),r#"{"id":"demo","title":"Demo","license":"MIT","images":[]}"#).unwrap();let out=d.path().join("demo.s9pk");build_s9pk(d.path(),&out).unwrap();let m=parse_manifest(&out).unwrap();assert_eq!(m.id,"demo");assert_ne!(compute_merkle_root(&out).unwrap(),[0;32]);}
 #[test] fn sign_verify(){let d=tempdir().unwrap();fs::create_dir_all(d.path().join("startos/manifest")).unwrap();fs::write(d.path().join("startos/manifest/manifest.json"),r#"{"id":"demo","title":"Demo","license":"MIT"}"#).unwrap();let out=d.path().join("demo.s9pk");build_s9pk(d.path(),&out).unwrap();let sk=SigningKey::generate(&mut OsRng);sign_s9pk(&out,sk.to_bytes().as_slice()).unwrap();verify_s9pk(&out,sk.verifying_key().as_bytes()).unwrap();}
 #[test] fn partial_range(){let d=tempdir().unwrap();fs::create_dir_all(d.path().join("startos/manifest")).unwrap();fs::write(d.path().join("startos/manifest/manifest.json"),r#"{"id":"demo","title":"Demo","license":"MIT"}"#).unwrap();let out=d.path().join("demo.s9pk");build_s9pk(d.path(),&out).unwrap();assert!(!extract_partial(&out,0..1).unwrap().is_empty());}
}