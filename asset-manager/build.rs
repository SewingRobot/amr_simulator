fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Uncomment when proto files are ready
    // tonic_build::configure()
    //     .build_server(true)
    //     .build_client(false)
    //     .compile_protos(
    //         &["../proto/asset/asset_service.proto"],
    //         &["../proto"],
    //     )?;
    Ok(())
}
