// 10/16/19
fn main() -> Result<(), Box<dyn std::error::Error>>  {
//    tonic_build::compile_protos("proto/helloworld.proto")?;
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/api/proto")
        .compile(
            &["proto/helloworld.proto"],
            &["proto"],
        )?;
    Ok(())
}