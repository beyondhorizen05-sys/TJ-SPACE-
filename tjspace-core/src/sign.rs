use anyhow::{anyhow,Result};
use base64::{engine::general_purpose::STANDARD,Engine};
use ed25519_dalek::{Signature,SigningKey,Signer,VerifyingKey,Verifier};

fn decode32(value:&[u8])->Result<[u8;32]>{
    let raw=if value.len()==32{value.to_vec()}else{STANDARD.decode(value)?};
    raw.try_into().map_err(|_|anyhow!("key must be exactly 32 bytes"))
}
pub fn sign(bytes:&[u8],key:&[u8])->Result<Vec<u8>>{
    let sk=SigningKey::from_bytes(&decode32(key)?);
    let sig=sk.sign(bytes);
    let mut out=Vec::with_capacity(96);
    out.extend_from_slice(sk.verifying_key().as_bytes());
    out.extend_from_slice(&sig.to_bytes());
    Ok(out)
}
pub fn verify(bytes:&[u8],signed_artifact:&[u8])->Result<()>{
    if signed_artifact.len()!=96{return Err(anyhow!("signed artifact must contain 32-byte public key and 64-byte signature"))}
    let vk=VerifyingKey::from_bytes(&signed_artifact[..32].try_into().unwrap())?;
    let sig=Signature::from_slice(&signed_artifact[32..])?;
    vk.verify(bytes,&sig).map_err(|e|anyhow!("signature invalid: {e}"))
}
