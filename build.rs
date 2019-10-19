// 10/16/19
const OUT_DIR: &str = "src/api/proto";

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(OUT_DIR)
        .compile(
            &["proto/api.proto", "proto/apigrpc.proto"],
            &["proto", "/usr/local/include"],
        )?;
    Ok(())
}