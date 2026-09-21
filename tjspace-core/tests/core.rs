use tjspace_core::{registry,sign,s9pk,tunnel};
use ed25519_dalek::SigningKey;
use rand_core::OsRng;

#[test]fn registry_accepts_http_https(){assert!(registry::validate(&registry::RegistryConfig{url:"https://example.test".into(),enabled:true}).is_ok());assert!(registry::validate(&registry::RegistryConfig{url:"ftp://example.test".into(),enabled:true}).is_err());}
#[test]fn tunnel_rejects_whitespace(){assert!(tunnel::validate_endpoint("127.0.0.1:1").is_ok());assert!(tunnel::validate_endpoint("bad endpoint").is_err());}
#[test]fn s9pk_reads_manifest(){
    let dir=tempfile::tempdir().unwrap();let p=dir.path().join("x.s9pk");let f=std::fs::File::create(&p).unwrap();let mut a=tar::Builder::new(f);
    let data=br#"{"package_id":"hello","version":"1.2.3"}"#;let mut h=tar::Header::new_gnu();h.set_path("manifest.json").unwrap();h.set_size(data.len() as u64);h.set_cksum();a.append(&h,&data[..]).unwrap();a.finish().unwrap();
    assert_eq!(s9pk::read_manifest(p.to_str().unwrap()).unwrap().package_id,"hello");
}
#[test]fn ed25519_roundtrip(){let sk=SigningKey::generate(&mut OsRng);let msg=b"tjspace";let sig=sign::sign(msg,sk.as_bytes()).unwrap();assert!(sign::verify(msg,&sig).is_ok());}
#[test]fn ed25519_rejects_modified_bytes(){let sk=SigningKey::generate(&mut OsRng);let sig=sign::sign(b"a",sk.as_bytes()).unwrap();assert!(sign::verify(b"b",&sig).is_err());}
