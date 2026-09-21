fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fds = protox::compile(["proto/state_sync.proto"], ["proto"])?;
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_fds(fds)?;
    println!("cargo:rerun-if-changed=proto/state_sync.proto");
    Ok(())
}
