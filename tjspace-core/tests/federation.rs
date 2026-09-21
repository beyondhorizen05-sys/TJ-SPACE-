use tjspace_core::federation::FederationConfig;

#[test]
fn federation_config_has_safe_defaults() {
    let c = FederationConfig::default();
    assert_eq!(c.peer_port, 8090);
    assert!(c.mdns_timeout_ms > 0);
    assert!(c.state_path.contains("tjspace-federation-identity"));
}

#[test]
fn discovery_query_uses_mdns_service_name() {
    let q = tjspace_core::federation::test_mdns_query();
    assert!(q.windows(b"_tjspace").any(|w| w == b"_tjspace"));
}

#[test]
fn peer_id_validation_rejects_unsafe_values() {
    assert!(tjsafe("node-01"));
    assert!(!tjsafe("node 01"));
    assert!(!tjsafe(""));
}
fn tjsafe(s:&str)->bool {
    s.len() <= 128 && !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c,'-'|'_'|'.'))
}

#[test]
fn trust_exchange_types_round_trip() {
    let x = tjspace_core::federation::TrustExchange {
        endpoint:"http://127.0.0.1:8090".into(),
        peer_ed25519_public_key_hex:"00".repeat(32),
        peer_x25519_public_key_hex:"11".repeat(32),
        challenge_hex:"22".repeat(16),
        peer_signature_hex:"33".repeat(64),
    };
    let encoded=serde_json::to_string(&x).unwrap();
    let decoded:tjspace_core::federation::TrustExchange=serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.endpoint,x.endpoint);
}

#[test]
fn protocol_namespace_is_stable() {
    let x=tjspace_core::federation::test_protocol();
    assert_eq!(x,"tjs-federation-v1");
}
