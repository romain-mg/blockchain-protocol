/// Compiles protocol buffer code using [`tonic_build`].
fn main() {
    // Ignore errors (the directory may already exist)
    let _ = std::fs::create_dir("src/rpc");

    tonic_build::configure()
        .out_dir("src/rpc")
        .compile_protos(
            &["proto/p2p.proto", "proto/clientToNode.proto"],
            &["proto/"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));
}
