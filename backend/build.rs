fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Uncomment when proto files are ready
    // tonic_build::configure()
    //     .build_server(false)
    //     .build_client(true)
    //     .compile_protos(
    //         &["../proto/simulation/sim_service.proto"],
    //         &["../proto"],
    //     )?;
    Ok(())
}
