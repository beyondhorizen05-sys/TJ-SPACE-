use tjspace_core::federation::{FederationConfig, Peer, TrustExchange, TrustOffer};

#[test]
fn federation_config_has_safe_defaults() {
    let c = FederationConfig::default();
    assert_eq!(c.peer_port, 8090);
    assert!(c.mdns_timeout_ms > 0);
    assert!(c.state_path.contains("tjspace-federation-identity"));
}

#[test]
fn trust_exchange_serializes() {
    let x = TrustExchange {
        endpoint:"http://127.0.0.1:8090".into(),
        peer_ed25519_public_key_hex:"00".repeat(32),
        peer_x25519_public_key_hex:"11".repeat(32),
        challenge_hex:"22".repeat(16),
        peer_signature_hex:"33".repeat(64),
    };
    let decoded:TrustExchange=serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
    assert_eq!(decoded.endpoint,x.endpoint);
    assert_eq!(decoded.challenge_hex.len(),32);
}

#[test]
fn trust_offer_round_trips() {
    let x=TrustOffer{peer_id:"peer-a".into(),endpoint:"https://peer.local:8090".into(),ed25519_public_key_hex:"00".repeat(32),x25519_public_key_hex:"11".repeat(32),challenge_hex:"22".repeat(16),signature_hex:"33".repeat(64)};
    let decoded:TrustOffer=serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
    assert_eq!(decoded.peer_id,"peer-a");
    assert_eq!(decoded.signature_hex.len(),128);
}

#[test]
fn peer_defaults_are_explicit() {
    let x=Peer{peer_id:"peer-a".into(),endpoint:"http://127.0.0.1:8090".into(),discovered:true,trusted:false,ed25519_public_key_hex:None,x25519_public_key_hex:None,last_seen_unix:0,registry_scopes:vec![]};
    assert!(x.discovered);
    assert!(!x.trusted);
    assert!(x.registry_scopes.is_empty());
}

#[test]
fn federation_config_yaml_round_trips() {
    let c=FederationConfig::default();
    let y=serde_yaml::to_string(&c).unwrap();
    let decoded:FederationConfig=serde_yaml::from_str(&y).unwrap();
    assert_eq!(decoded.peer_port,c.peer_port);
}
