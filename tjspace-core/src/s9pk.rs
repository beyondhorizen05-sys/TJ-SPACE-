use anyhow::{anyhow,Result};
use serde::Deserialize;
use std::{fs::File,path::Path};
use flate2::read::GzDecoder;
use tar::Archive;

#[derive(Debug,Clone,Deserialize)]
pub struct Manifest{
    pub package_id:String,
    pub version:String,
    #[serde(default)] pub service_name:Option<String>
}
pub fn read_manifest(path:&str)->Result<Manifest>{
    if !Path::new(path).is_file(){return Err(anyhow!("s9pk file not found"))}
    let file=File::open(path)?;
    let reader:Box<dyn std::io::Read>=if path.ends_with(".gz"){Box::new(GzDecoder::new(file))}else{Box::new(file)};
    let mut archive=Archive::new(reader);
    for entry in archive.entries()?{
        let mut e=entry?;
        if e.path()?.to_string_lossy()=="manifest.json"{
            let mut bytes=Vec::new();
            std::io::Read::read_to_end(&mut e,&mut bytes)?;
            return Ok(serde_json::from_slice(&bytes)?)
        }
    }
    Err(anyhow!("manifest.json missing from s9pk"))
}
